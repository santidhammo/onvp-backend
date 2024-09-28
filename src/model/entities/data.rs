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

use crate::Error;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use base64::engine::general_purpose;
use base64::Engine;
use diesel::{Identifiable, Queryable, Selectable};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Clone, Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberEntity {
    #[serde(default)]
    pub id: i32,

    #[schema(example = 1)]
    pub member_details_id: i32,

    #[schema(example = 1)]
    pub member_address_details_id: i32,

    #[schema(example = 1)]
    pub musical_instrument_id: Option<i32>,

    #[schema(example = "xyz.png")]
    pub picture_asset_id: Option<String>,

    #[serde(default)]
    pub activated: bool,

    #[serde(skip_serializing)]
    pub creation_time: chrono::NaiveDateTime,

    #[serde(skip, default)]
    pub activation_string: String,

    #[serde(skip, default)]
    pub activation_time: chrono::NaiveDateTime,

    #[schema(example = false)]
    pub allow_privacy_info_sharing: bool,

    #[serde(skip, default)]
    pub nonce: String,
}

impl MemberEntity {
    pub fn decoded_nonce(&self) -> Result<GenericArray<u8, U12>, Error> {
        let decoded = general_purpose::STANDARD.decode(&self.nonce)?;

        let buffer: [u8; 12] = decoded[..].try_into().map_err(|_| {
            Error::insufficient_bytes("Not enough bytes available in base64 decoded Nonce")
        })?;
        GenericArray::try_from(buffer)
            .map_err(|_| Error::insufficient_bytes("Not enough decoded bytes in Nonce"))
    }
}

#[derive(Serialize, ToSchema, Clone, Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::workgroups)]
pub struct WorkgroupEntity {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "Foo Group")]
    pub name: String,
}

#[derive(Serialize, ToSchema, Clone, Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_details)]
pub struct MemberDetailEntity {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,
}

impl MemberDetailEntity {
    pub(crate) fn name(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.first_name);
        s.push_str(" ");
        s.push_str(&self.last_name);
        s
    }
}

#[derive(Serialize, ToSchema, Clone, Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_address_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberAddressDetailEntity {
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
