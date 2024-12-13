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

use crate::generic::lazy::{FIRST_OPERATOR_ACTIVATION_MINUTES, MEMBER_ACTIVATION_MINUTES};
use crate::generic::security::generate_activation_string;
use crate::model::interface::commands::{
    FirstOperatorRegisterCommand, MemberRegisterCommand, MemberUpdateAddressCommand,
    MemberUpdateCommand, MemberUpdatePrivacyInfoSharingCommand,
};
use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail};
use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm};
use base64::engine::general_purpose;
use base64::Engine;
use std::ops::Add;

#[derive(Clone, Debug)]
pub struct ExtendedMember {
    pub id: i32,
    pub musical_instrument_id: Option<i32>,
    pub picture_asset_id: Option<String>,
    pub activated: bool,
    pub creation_time: chrono::NaiveDateTime,
    pub activation_string: String,
    pub activation_time: chrono::NaiveDateTime,
    pub allow_privacy_info_sharing: bool,
    pub nonce: String,
    pub member_detail: MemberDetail,
    pub member_address_detail: MemberAddressDetail,
    pub description: Option<String>,
}

impl Default for ExtendedMember {
    fn default() -> Self {
        Self {
            id: 0,
            musical_instrument_id: None,
            picture_asset_id: None,
            activated: false,
            creation_time: chrono::NaiveDateTime::default(),
            activation_string: "".to_owned(),
            activation_time: chrono::NaiveDateTime::default(),
            allow_privacy_info_sharing: false,
            nonce: "".to_owned(),
            member_detail: MemberDetail {
                id: 0,
                first_name: "".to_owned(),
                last_name: "".to_owned(),
                email_address: "".to_owned(),
                phone_number: "".to_owned(),
            },
            member_address_detail: MemberAddressDetail {
                id: 0,
                street: "".to_owned(),
                house_number: 0,
                house_number_postfix: None,
                postal_code: "".to_owned(),
                domicile: "".to_owned(),
            },
            description: None,
        }
    }
}

impl ExtendedMember {
    fn generate_encoded_nonce() -> String {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        general_purpose::STANDARD.encode(&nonce)
    }
}

impl From<(&Member, &MemberDetail, &MemberAddressDetail)> for ExtendedMember {
    fn from(
        (member, member_detail, member_address_detail): (
            &Member,
            &MemberDetail,
            &MemberAddressDetail,
        ),
    ) -> Self {
        Self {
            id: member.id,
            musical_instrument_id: member.musical_instrument_id,
            picture_asset_id: member.picture_asset_id.clone(),
            activated: member.activated,
            creation_time: member.creation_time,
            activation_string: member.activation_string.clone(),
            activation_time: member.activation_time,
            allow_privacy_info_sharing: member.allow_privacy_info_sharing,
            nonce: member.nonce.clone(),
            member_detail: member_detail.clone(),
            member_address_detail: member_address_detail.clone(),
            description: member.description.clone(),
        }
    }
}

/// Performs a GDPR scrubbed implementation to extract information from the database
impl From<(&Member, &MemberDetail)> for ExtendedMember {
    fn from((member, member_detail): (&Member, &MemberDetail)) -> Self {
        Self {
            id: member.id,
            musical_instrument_id: member.musical_instrument_id,
            picture_asset_id: member.picture_asset_id.clone(),
            activated: member.activated,
            creation_time: member.creation_time,
            activation_string: member.activation_string.clone(),
            activation_time: member.activation_time,
            allow_privacy_info_sharing: member.allow_privacy_info_sharing,
            nonce: "".to_owned(),
            // Scrub the member detail from possible GDPR data
            member_detail: MemberDetail {
                id: member_detail.id,
                first_name: member_detail.first_name.clone(),
                last_name: member_detail.last_name.clone(),
                email_address: "".to_owned(),
                phone_number: "".to_owned(),
            },
            member_address_detail: MemberAddressDetail::gdpr_fake(),
            description: member.description.clone(),
        }
    }
}

impl From<&MemberRegisterCommand> for ExtendedMember {
    fn from(value: &MemberRegisterCommand) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0, // Skipped during creation
            member_detail: MemberDetail::from(&value.detail_register_sub_command),
            member_address_detail: MemberAddressDetail::from(&value.address_register_sub_command),
            musical_instrument_id: None,
            picture_asset_id: None,
            activated: false,
            creation_time: now.naive_utc(),
            activation_string: generate_activation_string(),
            activation_time: now.add(*MEMBER_ACTIVATION_MINUTES).naive_utc(),
            allow_privacy_info_sharing: false,
            nonce: Self::generate_encoded_nonce(),
            description: None,
        }
    }
}

impl From<&FirstOperatorRegisterCommand> for ExtendedMember {
    fn from(value: &FirstOperatorRegisterCommand) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0, // Skipped during creation
            member_detail: MemberDetail::from(&value.detail_register_sub_command),
            member_address_detail: MemberAddressDetail::from(&value.address_register_sub_command),
            musical_instrument_id: None,
            picture_asset_id: None,
            activated: false,
            creation_time: now.naive_utc(),
            activation_string: generate_activation_string(),
            activation_time: now.add(*FIRST_OPERATOR_ACTIVATION_MINUTES).naive_utc(),
            allow_privacy_info_sharing: false,
            nonce: Self::generate_encoded_nonce(),
            description: None,
        }
    }
}

/// Merges the update command into an existing extended member
impl From<(&ExtendedMember, &MemberUpdateCommand)> for ExtendedMember {
    fn from((origin, command): (&ExtendedMember, &MemberUpdateCommand)) -> Self {
        let mut cloned = origin.clone();
        cloned.member_detail.first_name = command.first_name.clone();
        cloned.member_detail.last_name = command.last_name.clone();
        cloned.member_detail.email_address = command.email_address.clone();
        cloned.member_detail.phone_number = command.phone_number.clone();
        cloned.musical_instrument_id = command.musical_instrument_id;
        cloned.description = command.description.clone();
        cloned
    }
}

/// Merges the update command into an existing extended member
impl From<(&ExtendedMember, &MemberUpdateAddressCommand)> for ExtendedMember {
    fn from((origin, command): (&ExtendedMember, &MemberUpdateAddressCommand)) -> Self {
        let mut cloned = origin.clone();
        cloned.member_address_detail.street = command.street.clone();
        cloned.member_address_detail.house_number = command.house_number.clone();
        cloned.member_address_detail.house_number_postfix = command.house_number_postfix.clone();
        cloned.member_address_detail.postal_code = command.postal_code.clone();
        cloned.member_address_detail.domicile = command.domicile.clone();
        cloned
    }
}

/// Merges the update command into an existing extended member
impl From<(&ExtendedMember, &MemberUpdatePrivacyInfoSharingCommand)> for ExtendedMember {
    fn from((origin, command): (&ExtendedMember, &MemberUpdatePrivacyInfoSharingCommand)) -> Self {
        let mut cloned = origin.clone();
        cloned.allow_privacy_info_sharing = command.allow;
        cloned
    }
}

/// A facebook member is a very GDPR scrubbed set of member information meant for public usage
#[derive(Clone, Debug)]
pub struct FacebookMember {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub musical_instrument_id: Option<i32>,
    pub picture_asset_id: Option<String>,
    pub allow_privacy_info_sharing: bool,
    pub description: Option<String>,
}

impl From<(&Member, &MemberDetail)> for FacebookMember {
    fn from((member, member_detail): (&Member, &MemberDetail)) -> Self {
        Self {
            id: member.id,
            first_name: member_detail.first_name.clone(),
            last_name: member_detail.last_name.clone(),
            musical_instrument_id: member.musical_instrument_id,
            picture_asset_id: member.picture_asset_id.clone(),
            allow_privacy_info_sharing: member.allow_privacy_info_sharing,
            description: member.description.clone(),
        }
    }
}
