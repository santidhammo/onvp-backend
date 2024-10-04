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
use crate::model::storage;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberResponse {
    #[serde(default)]
    pub id: i32,

    #[schema(example = 1)]
    pub musical_instrument_id: Option<i32>,

    #[schema(example = "xyz.png")]
    pub picture_asset_id: Option<String>,

    #[serde(default)]
    pub activated: bool,

    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,
}

impl From<&storage::extended_entities::ExtendedMember> for MemberResponse {
    fn from(value: &storage::extended_entities::ExtendedMember) -> Self {
        Self {
            id: value.id,
            musical_instrument_id: value.musical_instrument_id,
            picture_asset_id: value.picture_asset_id.clone(),
            activated: value.activated,
            first_name: value.member_detail.first_name.clone(),
            last_name: value.member_detail.last_name.clone(),
            email_address: value.member_detail.email_address.clone(),
            phone_number: value.member_detail.phone_number.clone(),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberAddressResponse {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "Orchestra Street")]
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

impl From<&storage::extended_entities::ExtendedMember> for MemberAddressResponse {
    fn from(value: &storage::extended_entities::ExtendedMember) -> Self {
        Self {
            id: value.id,
            street: value.member_address_detail.street.clone(),
            house_number: value.member_address_detail.house_number,
            house_number_postfix: value.member_address_detail.house_number_postfix.clone(),
            postal_code: value.member_address_detail.postal_code.clone(),
            domicile: value.member_address_detail.domicile.clone(),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkgroupResponse {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "Orchestra Committee")]
    pub name: String,
}

impl From<&storage::entities::Workgroup> for WorkgroupResponse {
    fn from(value: &storage::entities::Workgroup) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
        }
    }
}
