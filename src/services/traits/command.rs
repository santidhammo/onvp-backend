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
use crate::generic::result::BackendResult;
use crate::model::interface::commands::{
    AssociateRoleCommand, DissociateRoleCommand, MemberRegisterCommand,
};

/// Controls actions which can be performed on member data
pub trait MemberCommandService {
    /// Registers a new member which is not activated yet, by supplying the command received from
    /// the interface.
    fn register_inactive(&self, command: &MemberRegisterCommand) -> BackendResult<i32>;
}

/// Controls actions which can be performed to manage roles
pub trait RoleCommandService {
    /// Associates a role
    fn associate_role(&self, command: &AssociateRoleCommand) -> BackendResult<()>;

    /// Dissociates a role
    fn dissociate_role(&self, command: &DissociateRoleCommand) -> BackendResult<()>;
}
