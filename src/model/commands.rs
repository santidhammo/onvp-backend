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

use diesel::Insertable;
use serde::Deserialize;
use utoipa::ToSchema;

/// Command to register a new entity
#[derive(Deserialize, ToSchema, Clone, Debug, Insertable)]
#[serde(rename_all = "camelCase")]
#[schema(as = WorkgroupRegisterCommand)]
#[diesel(table_name = crate::schema::workgroups)]
pub struct WorkgroupRegisterCommand {
    #[schema(example = "Foo Group")]
    pub name: String,
}

/// To register a new member, registration data is necessary. The registration data consists
/// of the data necessary to create the member itself, alongside the member details and member
/// address details.
#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberRegisterCommand {
    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,

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

#[derive(Deserialize, ToSchema, Clone, Debug, Insertable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_details)]
pub struct MemberDetailRegisterSubCommand {
    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug, Insertable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_address_details)]
pub struct AddressDetailRegisterSubCommand {
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

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirstOperatorRegisterCommand {
    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,

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
}
