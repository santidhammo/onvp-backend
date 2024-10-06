/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::generic::lazy::TOKEN_EXPIRY_HIGH_WATER_MARK;
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::{DatabaseConnection, DatabaseConnectionPool};
use crate::generic::Injectable;
use crate::model::interface::client::UserClaims;
use crate::model::interface::requests::AuthorizationRequest;
use crate::model::interface::responses::{AuthorizationResponse, MemberResponse};
use crate::repositories::definitions::{AuthorizationRepository, MemberRepository};
use crate::services::definitions::request::AuthorizationRequestService;
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::web::Data;
use chrono::{TimeDelta, Utc};
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use jwt_compact::alg::Ed25519;
use jwt_compact::UntrustedToken;
use log::info;
use r2d2::PooledConnection;
use std::ops::Add;
use std::sync::Arc;
use totp_rs::TOTP;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    member_repository: Data<dyn MemberRepository>,
    authorization_repository: Data<dyn AuthorizationRepository>,
    token_signer: Data<TokenSigner<UserClaims, Ed25519>>,
}

impl AuthorizationRequestService for Implementation {
    fn login(&self, login_data: &AuthorizationRequest) -> BackendResult<AuthorizationResponse> {
        let mut conn = self.pool.get()?;
        conn.transaction::<AuthorizationResponse, BackendError, _>(|conn| {
            let extended_member = self
                .member_repository
                .find_extended_by_email_address(conn, &login_data.email_address)?;

            let totp: TOTP = MemberResponse::from(&extended_member).try_into()?;
            let is_current = totp
                .check_current(&login_data.token)
                .map_err(|_| BackendError::forbidden())?;

            if is_current {
                let user_claims = UserClaims {
                    email_address: login_data.email_address.clone(),
                    roles: self
                        .authorization_repository
                        .find_composite_roles_by_member_id(conn, extended_member.id)?,
                };

                let mut access_cookie = self.token_signer.create_access_cookie(&user_claims)?;
                let mut refresh_cookie = self.token_signer.create_refresh_cookie(&user_claims)?;
                access_cookie.set_same_site(SameSite::Strict);
                refresh_cookie.set_same_site(SameSite::Strict);
                let cookies = vec![access_cookie, refresh_cookie];

                Ok(AuthorizationResponse {
                    member: MemberResponse::from(&extended_member),
                    composite_roles: user_claims.roles,
                    cookies,
                })
            } else {
                Err(BackendError::forbidden())
            }
        })
    }

    fn refresh(
        &self,
        client_user_claims: &UserClaims,
        access_cookie: &Cookie<'static>,
        refresh_cookie: &Cookie<'static>,
    ) -> BackendResult<AuthorizationResponse> {
        let mut conn = self.pool.get()?;
        conn.transaction::<AuthorizationResponse, BackendError, _>(|conn| {
            // Convert cookies to the associated tokens. Verification is already done at this point in time,
            // it is only necessary to refresh the situation appropriately.
            let origin_access_token = UntrustedToken::new(access_cookie.value())?;
            let origin_refresh_token = UntrustedToken::new(refresh_cookie.value())?;
            // If the refresh token nearly expires, the login procedure is transparently performed, to
            // ensure that user roles are still the same. If the access token nearly expires, then a new
            // access token is simply created, otherwise nothing is done.
            let (new_user_claims, new_cookies) =
                if Self::token_nearly_expires(origin_refresh_token)? {
                    info!(
                        "Refreshing tokens for member: {}",
                        &client_user_claims.email_address
                    );
                    let new_user_claims = self.reset_authority(&client_user_claims, conn)?;
                    let mut access_cookie =
                        self.token_signer.create_access_cookie(&new_user_claims)?;
                    let mut refresh_cookie =
                        self.token_signer.create_refresh_cookie(&new_user_claims)?;
                    access_cookie.set_same_site(SameSite::Strict);
                    refresh_cookie.set_same_site(SameSite::Strict);
                    (new_user_claims, vec![access_cookie, refresh_cookie])
                } else if Self::token_nearly_expires(origin_access_token)? {
                    let mut access_cookie = self
                        .token_signer
                        .create_access_cookie(&client_user_claims)?;
                    access_cookie.set_same_site(SameSite::Strict);
                    (client_user_claims.clone(), vec![access_cookie])
                } else {
                    (client_user_claims.clone(), vec![])
                };

            let extended_member = self
                .member_repository
                .find_extended_by_email_address(conn, &client_user_claims.email_address)?;

            Ok(AuthorizationResponse {
                member: MemberResponse::from(&extended_member),
                composite_roles: new_user_claims.roles,
                cookies: new_cookies,
            })
        })
    }

    fn logout(&self) -> BackendResult<Vec<Cookie<'static>>> {
        let access_cookie = Cookie::build("access_token".to_string(), "")
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(Expiration::DateTime(OffsetDateTime::UNIX_EPOCH))
            .finish();

        let refresh_cookie = Cookie::build("refresh_token".to_string(), "")
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(Expiration::DateTime(OffsetDateTime::UNIX_EPOCH))
            .finish();

        Ok(vec![access_cookie, refresh_cookie])
    }
}

impl
    Injectable<
        (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn AuthorizationRepository>,
            &Data<TokenSigner<UserClaims, Ed25519>>,
        ),
        dyn AuthorizationRequestService,
    > for Implementation
{
    fn injectable(
        (pool, member_repository, authorization_repository, token_signer): (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn AuthorizationRepository>,
            &Data<TokenSigner<UserClaims, Ed25519>>,
        ),
    ) -> Data<dyn AuthorizationRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
            authorization_repository: authorization_repository.clone(),
            token_signer: token_signer.clone(),
        };
        let arc: Arc<dyn AuthorizationRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}

impl Implementation {
    fn token_nearly_expires(token: UntrustedToken) -> BackendResult<bool> {
        let expiration = token
            .deserialize_claims_unchecked::<UserClaims>()?
            .expiration
            .ok_or(BackendError::bad())?;
        let delta = TimeDelta::seconds((*TOKEN_EXPIRY_HIGH_WATER_MARK) as i64);
        let high_water_mark = expiration.add(-delta);
        Ok(high_water_mark.le(&Utc::now()))
    }

    fn reset_authority(
        &self,
        user_claims: &&UserClaims,
        conn: &mut PooledConnection<ConnectionManager<DatabaseConnection>>,
    ) -> Result<UserClaims, BackendError> {
        let user_claims = conn.transaction::<UserClaims, BackendError, _>(|conn| {
            let extended_member = self
                .member_repository
                .find_extended_by_email_address(conn, &user_claims.email_address)?;
            let user_claims = UserClaims {
                email_address: user_claims.email_address.clone(),
                roles: self
                    .authorization_repository
                    .find_composite_roles_by_member_id(conn, extended_member.id)?,
            };
            Ok(user_claims)
        })?;
        Ok(user_claims)
    }
}
