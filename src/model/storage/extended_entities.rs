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
    MemberUpdateCommand,
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