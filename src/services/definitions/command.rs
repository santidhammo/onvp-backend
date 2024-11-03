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
    AssociateMemberToWorkgroupCommand, AssociateRoleCommand, CreatePageCommand,
    DissociateMemberFromWorkgroupCommand, DissociateRoleCommand, FirstOperatorRegisterCommand,
    ImageUploadCommand, MemberActivationCommand, MemberRegisterCommand, MemberUpdateAddressCommand,
    MemberUpdateCommand, MemberUpdatePrivacyInfoSharingCommand, PublishPageCommand,
    UpdatePageCommand, WorkgroupRegisterCommand, WorkgroupUpdateCommand,
};

/// Controls actions which can be performed on member data
pub trait MemberCommandService {
    /// Registers a new member which is not activated yet, by supplying the command received from
    /// the interface.
    fn register_inactive(&self, command: &MemberRegisterCommand) -> BackendResult<i32>;

    /// Updates the regular details of an existing member
    fn update(&self, member_id: i32, command: &MemberUpdateCommand) -> BackendResult<()>;

    /// Updates the address details of an existing member
    fn update_address(
        &self,
        member_id: i32,
        command: &MemberUpdateAddressCommand,
    ) -> BackendResult<()>;

    /// Updates whether the member allows for (weak) privacy related information to be shared
    fn update_privacy_info_sharing(
        &self,
        member_id: i32,
        command: &MemberUpdatePrivacyInfoSharingCommand,
    ) -> BackendResult<()>;

    /// Unregisters an existing member
    fn unregister(&self, member_id: i32) -> BackendResult<()>;
}
pub trait MemberPictureCommandService {
    fn upload(&self, member_id: i32, command: &ImageUploadCommand) -> BackendResult<String>;
}

/// Controls activation of members
pub trait MemberActivationCommandService {
    /// Activates a member based on the token data
    fn activate(&self, data: &MemberActivationCommand) -> BackendResult<()>;
}

/// Controls actions which can be performed on member data
pub trait SetupCommandService {
    /// Registers a new member which is not activated yet, by supplying the command received from
    /// the interface.
    fn register_first_operator(
        &self,
        command: &FirstOperatorRegisterCommand,
    ) -> BackendResult<String>;
}

/// Controls actions which can be performed to manage work groups
pub trait WorkgroupCommandService {
    /// Registers a new work group
    fn register(&self, command: &WorkgroupRegisterCommand) -> BackendResult<i32>;

    /// Updates an existing work group
    fn update(&self, workgroup_id: i32, command: &WorkgroupUpdateCommand) -> BackendResult<()>;

    /// Unregisters an existing work group
    fn unregister(&self, workgroup_id: i32) -> BackendResult<()>;

    /// Associates a member to a work group
    fn associate_member_to_workgroup(
        &self,
        command: &AssociateMemberToWorkgroupCommand,
    ) -> BackendResult<()>;

    ///Dissociates a member to a work group
    fn dissociate_member_from_workgroup(
        &self,
        command: &DissociateMemberFromWorkgroupCommand,
    ) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage roles
pub trait RoleCommandService {
    /// Associates a role
    fn associate_role(&self, command: &AssociateRoleCommand) -> BackendResult<()>;

    /// Dissociates a role
    fn dissociate_role(&self, command: &DissociateRoleCommand) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage pages
pub trait PageCommandService {
    /// Creates a new page
    fn create(&self, command: &CreatePageCommand) -> BackendResult<()>;

    /// Sets the content of a given page
    fn set_content(&self, page_id: i32, content: &str) -> BackendResult<()>;

    /// Updates a page
    fn update(&self, page_id: i32, command: &UpdatePageCommand) -> BackendResult<()>;

    /// Publishes the page
    fn publish(&self, page_id: i32, command: &PublishPageCommand) -> BackendResult<()>;
    fn unpublish(&self, page_id: i32) -> BackendResult<()>;

    /// Deletes an existing page
    fn delete(&self, page_id: i32) -> BackendResult<()>;
}
