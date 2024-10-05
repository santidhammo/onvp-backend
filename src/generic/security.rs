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

use crate::generic::result::{BackendError, BackendResult};
use crate::model::interface::client::UserClaims;
use crate::model::primitives::Role;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
pub use totp_rs::TOTP;

pub fn operator_state_guard(claims: &UserClaims) -> BackendResult<()> {
    if claims.has_role(Role::Operator) {
        Ok(())
    } else {
        Err(BackendError::bad())
    }
}

pub fn generate_activation_string() -> String {
    let validation_string = Alphanumeric.sample_string(&mut thread_rng(), 32);
    validation_string
}
