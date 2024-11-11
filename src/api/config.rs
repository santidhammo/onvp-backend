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
use crate::api::middleware::authority::config::AuthorityConfig;
use crate::api::middleware::authority::Allowance::{Any, LoggedInMember};
use crate::generic::http::Method::{Get, Post};

pub fn configure_authority() -> AuthorityConfig {
    AuthorityConfig::new()
        .allow(Get, "/docs", Any)
        .allow(Get, "/api/facebook/v1/**", Any)
        .allow(Get, "/api/setup/v1/**", Any)
        .allow(Post, "/api/setup/v1/**", Any)
        .allow(Post, "/api/authorization/v1/login", Any)
        .allow(Get, "/api/authorization/v1/logout", Any)
        .allow(Get, "/api/authorization/v1/refresh", LoggedInMember)
        .allow(Get, "/api/members/v1/activation/code/**", Any)
        .allow(Post, "/api/members/v1/activation/activate", Any)
        .allow(Get, "/api/members/v1/picture_asset", LoggedInMember)
        .allow(Get, "/api/members/v1/picture", LoggedInMember)
        .allow(Get, "/api/workgroups/v1/search", LoggedInMember)
        .allow(Get, "/api/workgroups/v1/**", LoggedInMember)
        .allow(Get, "/api/source_code_details/v1/**", Any)
        .allow(Get, "/api/pages/v1/main-menu", Any)
        .allow(Get, "/api/pages/v1/default", Any)
        .allow(Get, "/api/pages/v1/search", Any)
        .allow(Get, "/api/pages/v1/page/**", Any)
        .allow(Get, "/api/images/v1/image/**", Any)
        .allow(Get, "/api/images/v1/asset/**", Any)
}
