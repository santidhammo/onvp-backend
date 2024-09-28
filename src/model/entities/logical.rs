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

use crate::model::entities::data::{MemberDetailEntity, MemberEntity};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberWithDetailLogicalEntity {
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

impl From<&(MemberEntity, MemberDetailEntity)> for MemberWithDetailLogicalEntity {
    fn from((member, member_detail): &(MemberEntity, MemberDetailEntity)) -> Self {
        let member = member.clone();
        let member_detail = member_detail.clone();
        Self {
            id: member.id,
            musical_instrument_id: member.musical_instrument_id,
            picture_asset_id: member.picture_asset_id,
            activated: member.activated,
            first_name: member_detail.first_name,
            last_name: member_detail.last_name,
            email_address: member_detail.email_address,
            phone_number: member_detail.phone_number,
        }
    }
}
