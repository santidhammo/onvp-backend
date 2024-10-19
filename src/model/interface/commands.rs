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
use crate::model::interface::sub_commands::{AddressRegisterSubCommand, DetailRegisterSubCommand};
use crate::model::primitives::{Role, RoleClass};
use actix_web::web::Bytes;
use image::{DynamicImage, ImageReader};
use serde::Deserialize;
use std::io::Cursor;
use utoipa::ToSchema;

/// Command to register a new work group
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkgroupRegisterCommand {
    #[schema(example = "Orchestra Committee")]
    pub name: String,
}

/// Command to update an existing work group
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkgroupUpdateCommand {
    #[schema(example = "Orchestra Committee")]
    pub name: String,
}

/// To register a new member, registration data is necessary. The registration data consists
/// of the data necessary to create the member itself, alongside the member details and member
/// address details.
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberRegisterCommand {
    pub detail_register_sub_command: DetailRegisterSubCommand,
    pub address_register_sub_command: AddressRegisterSubCommand,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirstOperatorRegisterCommand {
    pub detail_register_sub_command: DetailRegisterSubCommand,
    pub address_register_sub_command: AddressRegisterSubCommand,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberUpdateCommand {
    #[schema(example = 1)]
    pub musical_instrument_id: Option<i32>,

    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,

    #[schema(example = "Describe something about this member")]
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberUpdatePrivacyInfoSharingCommand {
    #[schema(example = true)]
    pub allow: bool,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberUpdateAddressCommand {
    #[schema(example = "Orchestra Road")]
    pub street: String,

    #[schema(example = 1)]
    pub house_number: i32,

    #[schema(example = "a")]
    pub house_number_postfix: Option<String>,

    #[schema(example = "9999ZZ")]
    pub postal_code: String,

    #[schema(example = "Tubaton")]
    pub domicile: String,
}

/// Command to associate a member to a work group
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssociateMemberToWorkgroupCommand {
    #[schema(example = "1")]
    pub member_id: i32,

    #[schema(example = "1")]
    pub workgroup_id: i32,
}

/// Command to dissociate a member from a work group
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DissociateMemberFromWorkgroupCommand {
    #[schema(example = "1")]
    pub member_id: i32,

    #[schema(example = "1")]
    pub workgroup_id: i32,
}

/// Associates a class with a given identifier to a given role
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssociateRoleCommand {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "OrchestraCommittee")]
    pub role: Role,

    #[schema(example = "Member")]
    #[serde(rename = "roleClass")]
    pub class: RoleClass,
}

/// Dissociates a class with a given identifier from a given role
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DissociateRoleCommand {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "OrchestraCommittee")]
    pub role: Role,

    #[schema(example = "Member")]
    #[serde(rename = "roleClass")]
    pub class: RoleClass,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberActivationCommand {
    #[schema(example = "abc")]
    pub activation_string: String,
    #[schema(example = "123456")]
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct ImageUploadCommand {
    pub dynamic_image: DynamicImage,
}

impl TryFrom<&Bytes> for ImageUploadCommand {
    type Error = BackendError;

    fn try_from(value: &Bytes) -> BackendResult<Self> {
        Ok(Self {
            dynamic_image: ImageReader::new(Cursor::new(value))
                .with_guessed_format()?
                .decode()?,
        })
    }
}
