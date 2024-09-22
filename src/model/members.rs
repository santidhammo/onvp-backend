use crate::Error;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use base64::engine::general_purpose;
use base64::Engine;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    #[serde(default)]
    #[diesel(skip_insertion)]
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

impl Member {
    pub fn decoded_nonce(&self) -> Result<GenericArray<u8, U12>, Error> {
        let decoded = general_purpose::STANDARD.decode(&self.nonce)?;

        let buffer: [u8; 12] = decoded[..].try_into().map_err(|_| {
            Error::insufficient_bytes("Not enough bytes available in base64 decoded Nonce")
        })?;
        GenericArray::try_from(buffer)
            .map_err(|_| Error::insufficient_bytes("Not enough decoded bytes in Nonce"))
    }
}

#[derive(
    Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberDetail {
    #[serde(default)]
    #[diesel(skip_insertion)]
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

impl MemberDetail {
    pub(crate) fn name(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.first_name);
        s.push_str(" ");
        s.push_str(&self.last_name);
        s
    }
}

#[derive(
    Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_address_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberAddressDetail {
    #[serde(default)]
    #[diesel(skip_insertion)]
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

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberWithDetail {
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

impl From<&(Member, MemberDetail)> for MemberWithDetail {
    fn from((member, member_detail): &(Member, MemberDetail)) -> Self {
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

/// To register a new member, registration data is necessary. The registration data consists
/// of the data necessary to create the member itself, alongside the member details and member
/// address details.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberRegistrationData {
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
