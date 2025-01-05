/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024-2025.  Sjoerd van Leent
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
use crate::generic::storage::session::Session;
use crate::model::interface::commands::{
    AssociateMemberToWorkgroupCommand, AssociateRoleCommand, CreateMailTemplateCommand,
    CreatePageCommand, DissociateMemberFromWorkgroupCommand, DissociateRoleCommand,
    FirstOperatorRegisterCommand, ImageUploadCommand, MemberActivationCommand,
    MemberImageUploadCommand, MemberRegisterCommand, MemberUpdateAddressCommand,
    MemberUpdateCommand, MemberUpdatePrivacyInfoSharingCommand, PublishImageCommand,
    PublishPageCommand, RegisterMusicalInstrumentCommand, SendMailCommand,
    UpdateMailTemplateCommand, UpdateMusicalInstrumentCommand, UpdatePageCommand,
    WorkgroupRegisterCommand, WorkgroupUpdateCommand,
};

/// Controls actions which can be performed on member data
pub trait MemberCommandService {
    /// Registers a new member which is not activated yet, by supplying the command received from
    /// the interface.
    fn register_inactive(
        &self,
        session: Session,
        command: &MemberRegisterCommand,
    ) -> BackendResult<i32>;

    /// Updates the regular details of an existing member
    fn update(
        &self,
        session: Session,
        member_id: i32,
        command: &MemberUpdateCommand,
    ) -> BackendResult<()>;

    /// Updates the address details of an existing member
    fn update_address(
        &self,
        session: Session,
        member_id: i32,
        command: &MemberUpdateAddressCommand,
    ) -> BackendResult<()>;

    /// Updates whether the member allows for (weak) privacy related information to be shared
    fn update_privacy_info_sharing(
        &self,
        session: Session,
        member_id: i32,
        command: &MemberUpdatePrivacyInfoSharingCommand,
    ) -> BackendResult<()>;

    /// Unregisters an existing member
    fn unregister(&self, session: Session, member_id: i32) -> BackendResult<()>;
}
pub trait MemberPictureCommandService {
    fn upload(
        &self,
        session: Session,
        member_id: i32,
        command: &MemberImageUploadCommand,
    ) -> BackendResult<String>;
}

/// Controls activation of members
pub trait MemberActivationCommandService {
    /// Activates a member based on the token data
    fn activate(&self, session: Session, data: &MemberActivationCommand) -> BackendResult<()>;
}

/// Controls actions which can be performed on member data
pub trait SetupCommandService {
    /// Registers a new member which is not activated yet, by supplying the command received from
    /// the interface.
    fn register_first_operator(
        &self,
        session: Session,
        command: &FirstOperatorRegisterCommand,
    ) -> BackendResult<String>;
}

/// Controls actions which can be performed to manage work groups
pub trait WorkgroupCommandService {
    /// Registers a new work group
    fn register(&self, session: Session, command: &WorkgroupRegisterCommand) -> BackendResult<i32>;

    /// Updates an existing work group
    fn update(
        &self,
        session: Session,
        workgroup_id: i32,
        command: &WorkgroupUpdateCommand,
    ) -> BackendResult<()>;

    /// Unregisters an existing work group
    fn unregister(&self, session: Session, workgroup_id: i32) -> BackendResult<()>;

    /// Associates a member to a work group
    fn associate_member_to_workgroup(
        &self,
        session: Session,
        command: &AssociateMemberToWorkgroupCommand,
    ) -> BackendResult<()>;

    ///Dissociates a member to a work group
    fn dissociate_member_from_workgroup(
        &self,
        session: Session,
        command: &DissociateMemberFromWorkgroupCommand,
    ) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage roles
pub trait RoleCommandService {
    /// Associates a role
    fn associate_role(&self, session: Session, command: &AssociateRoleCommand)
        -> BackendResult<()>;

    /// Dissociates a role
    fn dissociate_role(
        &self,
        session: Session,
        command: &DissociateRoleCommand,
    ) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage pages
pub trait PageCommandService {
    /// Creates a new page
    fn create(&self, session: Session, command: &CreatePageCommand) -> BackendResult<()>;

    /// Sets the content of a given page
    fn set_content(&self, session: Session, page_id: i32, content: &str) -> BackendResult<()>;

    /// Updates a page
    fn update(
        &self,
        session: Session,
        page_id: i32,
        command: &UpdatePageCommand,
    ) -> BackendResult<()>;

    /// Publishes the page
    fn publish(
        &self,
        session: Session,
        page_id: i32,
        command: &PublishPageCommand,
    ) -> BackendResult<()>;

    /// Unpublishes the page
    fn unpublish(&self, session: Session, page_id: i32) -> BackendResult<()>;

    /// Deletes an existing page
    fn delete(&self, session: Session, page_id: i32) -> BackendResult<()>;

    /// Sets the default page
    fn set_default(&self, session: Session, page_id: i32) -> BackendResult<()>;

    /// Sets the order of the page
    fn set_order(&self, session: Session, page_id: i32, order_number: i32) -> BackendResult<()>;

    /// Sets the parent id of the page
    fn set_or_unset_parent_id(
        &self,
        session: Session,
        page_id: i32,
        maybe_parent_id: Option<i32>,
    ) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage images
pub trait ImageCommandService {
    /// Handles uploading a new image
    fn upload(&self, session: Session, command: &ImageUploadCommand) -> BackendResult<String>;

    /// Publishes the image
    fn publish(
        &self,
        session: Session,
        image_id: i32,
        command: &PublishImageCommand,
    ) -> BackendResult<()>;

    /// Unpublishes the image
    fn unpublish(&self, session: Session, image_id: i32) -> BackendResult<()>;

    /// Deletes an existing image
    fn delete(&self, session: Session, image_id: i32) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage musical instruments
pub trait MusicalInstrumentCommandService {
    /// Registers a new musical instrument
    fn register(
        &self,
        session: Session,
        command: &RegisterMusicalInstrumentCommand,
    ) -> BackendResult<()>;

    /// Updates a registered musical instrument
    fn update(
        &self,
        session: Session,
        musical_instrument_id: i32,
        command: &UpdateMusicalInstrumentCommand,
    ) -> BackendResult<()>;

    /// Deletes a registered musical instrument
    fn delete(&self, session: Session, musical_instrument_id: i32) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage email templates
pub trait MailTemplateCommandService {
    /// Creates a new email template
    fn create(&self, session: Session, command: &CreateMailTemplateCommand) -> BackendResult<()>;

    /// Updates a registered email template
    fn update(
        &self,
        session: Session,
        mail_template_id: i32,
        command: &UpdateMailTemplateCommand,
    ) -> BackendResult<()>;

    /// Deletes a registered email template
    fn delete(&self, session: Session, mail_template_id: i32) -> BackendResult<()>;
}

/// Controls actions which can be performed to manage mailings
pub trait MailingCommandService {
    /// Sends a new email
    fn send(&self, session: Session, command: &SendMailCommand) -> BackendResult<()>;
}
