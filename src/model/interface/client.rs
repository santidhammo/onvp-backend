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
use crate::model::primitives::Role;
use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, FromRequest)]
pub struct UserClaims {
    pub email_address: String,
    pub roles: Vec<Role>,
}

impl UserClaims {
    /// Checks if the user claims contain the given role
    pub fn has_role(&self, role: Role) -> bool {
        for intern in &self.roles {
            if intern == &role {
                return true;
            }
        }
        false
    }
}