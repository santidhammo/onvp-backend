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
use crate::model::interface::commands::WorkgroupRegisterCommand;
use crate::model::interface::sub_commands;
use crate::model::storage::extended_entities::ExtendedMember;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::members)]
pub struct Member {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub member_details_id: i32,
    pub member_address_details_id: i32,
    pub musical_instrument_id: Option<i32>,
    pub picture_asset_id: Option<String>,
    pub activated: bool,
    pub creation_time: chrono::NaiveDateTime,
    pub activation_string: String,
    pub activation_time: chrono::NaiveDateTime,
    pub allow_privacy_info_sharing: bool,
    pub nonce: String,
}

impl From<&ExtendedMember> for Member {
    fn from(value: &ExtendedMember) -> Self {
        Self {
            id: value.id,
            musical_instrument_id: value.musical_instrument_id,
            picture_asset_id: value.picture_asset_id.clone(),
            activated: value.activated,
            creation_time: value.creation_time,
            activation_string: value.activation_string.clone(),
            activation_time: value.activation_time,
            allow_privacy_info_sharing: value.allow_privacy_info_sharing,
            nonce: value.nonce.clone(),
            member_details_id: value.member_detail.id,
            member_address_details_id: value.member_address_detail.id,
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::member_details)]
pub struct MemberDetail {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub phone_number: String,
}

impl MemberDetail {
    pub(crate) fn name(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.first_name);
        s.push_str(" ");
        s.push_str(&self.last_name);
        s
    }
}

impl From<&sub_commands::DetailRegisterSubCommand> for MemberDetail {
    fn from(input: &sub_commands::DetailRegisterSubCommand) -> Self {
        Self {
            id: 0, // Skipped during creation
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            email_address: input.email_address.clone(),
            phone_number: input.phone_number.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::member_address_details)]
pub struct MemberAddressDetail {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub street: String,
    pub house_number: i32,
    pub house_number_postfix: Option<String>,
    pub postal_code: String,
    pub domicile: String,
}

impl From<&sub_commands::AddressRegisterSubCommand> for MemberAddressDetail {
    fn from(input: &sub_commands::AddressRegisterSubCommand) -> Self {
        Self {
            id: 0, // Skipped during creation
            street: input.street.clone(),
            house_number: input.house_number.clone(),
            house_number_postfix: input.house_number_postfix.clone(),
            postal_code: input.postal_code.clone(),
            domicile: input.domicile.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::workgroups)]
pub struct Workgroup {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
}

impl From<&WorkgroupRegisterCommand> for Workgroup {
    fn from(input: &WorkgroupRegisterCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            name: input.name.to_string(),
        }
    }
}
