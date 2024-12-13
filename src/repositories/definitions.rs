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
use crate::generic::security::ClaimRoles;
use crate::generic::storage::session::Session;
use crate::model::primitives::Role;
use crate::model::storage::entities::{Image, MailTemplate, MusicalInstrument, Page, Workgroup};
use crate::model::storage::extended_entities::{ExtendedMember, FacebookMember};

pub trait PropertiesRepository {
    fn maybe_int_property(&self, session: &mut Session, key: &str) -> Option<i32>;
    fn set_int_property(
        &self,
        session: &mut Session,
        key: &str,
        value: Option<i32>,
    ) -> BackendResult<()>;
}

pub trait MemberRepository {
    fn create_inactive(
        &self,
        session: &mut Session,
        member_extended: &ExtendedMember,
    ) -> BackendResult<i32>;

    fn find_extended_by_id(&self, session: &mut Session, id: i32) -> BackendResult<ExtendedMember>;

    fn find_extended_by_activation_string(
        &self,
        session: &mut Session,
        activation_string: &str,
    ) -> BackendResult<ExtendedMember>;

    fn find_extended_by_email_address(
        &self,
        session: &mut Session,
        email_address: &str,
    ) -> BackendResult<ExtendedMember>;

    fn find_workgroups(&self, session: &mut Session, id: i32) -> BackendResult<Vec<Workgroup>>;

    fn save(&self, session: &mut Session, member: ExtendedMember) -> BackendResult<()>;

    fn count_members_with_role(&self, session: &mut Session, role: Role) -> BackendResult<usize>;

    fn activate_by_id(&self, session: &mut Session, member_id: i32) -> BackendResult<()>;

    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)>;

    fn unregister(&self, session: &mut Session, member_id: i32) -> BackendResult<()>;
}

pub trait WorkgroupRepository {
    fn register(&self, session: &mut Session, workgroup: Workgroup) -> BackendResult<i32>;

    fn find_by_id(&self, session: &mut Session, id: i32) -> BackendResult<Workgroup>;

    fn save(&self, session: &mut Session, workgroup: Workgroup) -> BackendResult<()>;

    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<Workgroup>)>;

    fn unregister(&self, session: &mut Session, workgroup_id: i32) -> BackendResult<()>;

    fn find_members_by_id(
        &self,
        session: &mut Session,
        workgroup_id: i32,
    ) -> BackendResult<Vec<ExtendedMember>>;

    fn associate_member_to_workgroup(
        &self,
        session: &mut Session,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()>;
    fn dissociate_member_from_workgroup(
        &self,
        session: &mut Session,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()>;

    fn available_members_search(
        &self,
        session: &mut Session,
        workgroup_id: i32,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)>;
}

pub trait MemberPictureRepository {
    fn save_by_member_id(
        &self,
        session: &mut Session,
        member_id: i32,
        picture_asset_id: &str,
    ) -> BackendResult<()>;
}

pub trait MemberRoleRepository {
    /// Associates a member and a role
    fn associate(&self, session: &mut Session, member_id: i32, role: Role) -> BackendResult<()>;
    /// Dissociates a member from a role
    fn dissociate(&self, session: &mut Session, member_id: i32, role: Role) -> BackendResult<()>;
    /// Lists all roles of a specific member
    fn list_by_id(&self, session: &mut Session, member_id: i32) -> BackendResult<Vec<Role>>;
}

pub trait WorkgroupRoleRepository {
    /// Associates a work group and a role
    fn associate(&self, session: &mut Session, workgroup_id: i32, role: Role) -> BackendResult<()>;
    /// Dissociates a work group from a role
    fn dissociate(&self, session: &mut Session, workgroup_id: i32, role: Role)
        -> BackendResult<()>;
    /// Lists all roles of a specific workgroup
    fn list_by_id(&self, session: &mut Session, workgroup_id: i32) -> BackendResult<Vec<Role>>;
}

/// Manages a virtual view over the different roles from both members and associated work groups
pub trait AuthorizationRepository {
    /// Finds all roles for a member from direct association and work group association
    fn find_composite_roles_by_member_id(
        &self,
        session: &mut Session,
        member_id: i32,
    ) -> BackendResult<Vec<Role>>;
}

/// Manages a public repository for the face book, with a more minimalist amount of data
pub trait FacebookRepository {
    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<FacebookMember>)>;
}

/// Manages the page repository
pub trait PageRepository {
    /// Creates a new page and stores it into the database
    fn create(&self, session: &mut Session, page: Page) -> BackendResult<()>;

    /// Updates an existing page and stores it into the database
    fn update(&self, session: &mut Session, page: Page) -> BackendResult<()>;

    /// Finds the page by the identifier
    fn find_by_id(&self, session: &mut Session, page_id: i32) -> BackendResult<Page>;

    /// Lists the pages by the parent identifier
    fn list_by_parent_id(
        &self,
        session: &mut Session,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<Page>>;

    /// Finds the roles associated to a page
    fn find_associated_roles_by_id(
        &self,
        session: &mut Session,
        page_id: i32,
    ) -> BackendResult<Vec<Role>>;

    /// Removes a page by the identifier
    fn delete(&self, session: &mut Session, page_id: i32) -> BackendResult<()>;

    /// Drops the roles associated to a page
    fn reset_roles(&self, session: &mut Session, page_id: i32) -> BackendResult<()>;

    /// Assigns (and therefor publishes) the roles associated to a page (excluding the operator)
    fn assign_roles(
        &self,
        conn: &mut Session,
        page_id: i32,
        roles: &Vec<Role>,
    ) -> BackendResult<()>;

    /// Searches for all pages meeting any of the allowed roles
    fn search(
        &self,
        conn: &mut Session,
        page_offset: usize,
        term: &str,
        roles: &ClaimRoles,
    ) -> BackendResult<(usize, usize, Vec<Page>)>;
}

/// Manages the image repository
pub trait ImageRepository {
    /// Creates a new image and stores it into the database
    fn create(&self, session: &mut Session, image: Image) -> BackendResult<()>;

    /// Finds the image by the identifier
    fn find_by_id(&self, session: &mut Session, image_id: i32) -> BackendResult<Image>;

    /// Finds the roles associated to an image
    fn find_associated_roles_by_id(
        &self,
        session: &mut Session,
        image_id: i32,
    ) -> BackendResult<Vec<Role>>;

    /// Removes an image by the identifier
    fn delete(&self, session: &mut Session, image_id: i32) -> BackendResult<()>;

    /// Drops the roles associated to an image
    fn reset_roles(&self, session: &mut Session, image_id: i32) -> BackendResult<()>;

    /// Assigns (and therefor publishes) the roles associated to a page (excluding the operator)
    fn assign_roles(
        &self,
        session: &mut Session,
        image_id: i32,
        roles: &Vec<Role>,
    ) -> BackendResult<()>;

    /// Searches for all images meeting any of the allowed roles
    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<Image>)>;
}

/// Manages the musical instrument repository
pub trait MusicalInstrumentRepository {
    /// Creates a new musical instrument and stores it into the database
    fn create(&self, session: &mut Session, instrument: MusicalInstrument) -> BackendResult<()>;

    /// Updates an existing musical instrument in the database
    fn update(&self, session: &mut Session, instrument: MusicalInstrument) -> BackendResult<()>;

    /// Removes an existing musical instrument from the database
    fn delete(&self, session: &mut Session, instrument_id: i32) -> BackendResult<()>;

    /// Finds a musical instrument from the database using the identifier
    fn find_by_id(&self, session: &mut Session, image_id: i32) -> BackendResult<MusicalInstrument>;

    /// Searches for musical instruments matching with names matching the given term
    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<MusicalInstrument>)>;
}

/// Manages the email template repository
pub trait MailTemplateRepository {
    /// Creates a new email template and stores it into the database
    fn create(&self, session: &mut Session, instrument: MailTemplate) -> BackendResult<()>;

    /// Updates an existing email template in the database
    fn update(&self, session: &mut Session, instrument: MailTemplate) -> BackendResult<()>;

    /// Removes an existing email template from the database
    fn delete(&self, session: &mut Session, instrument_id: i32) -> BackendResult<()>;

    /// Finds an email template from the database using the identifier
    fn find_by_id(&self, session: &mut Session, image_id: i32) -> BackendResult<MailTemplate>;

    /// Lists all email templates stored in the databases
    fn list(&self, session: &mut Session) -> BackendResult<Vec<(i32, String)>>;
}
