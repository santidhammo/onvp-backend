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
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetailRegisterSubCommand {
    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressRegisterSubCommand {
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
