use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use base64::engine::general_purpose;
use base64::Engine;
use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    #[serde(skip_serializing, default)]
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

    #[serde(skip_serializing, default)]
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
    pub fn decoded_nonce(&self) -> Result<GenericArray<u8, U12>, String> {
        let decoded = general_purpose::STANDARD
            .decode(&self.nonce)
            .map_err(|e| e.to_string())?;

        let buffer: [u8; 12] = decoded[..]
            .try_into()
            .map_err(|_| "Not enough decoded bytes in Nonce".to_owned())?;
        GenericArray::try_from(buffer).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberDetails {
    #[serde(skip_serializing, default)]
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

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_address_details)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberAddressDetails {
    #[serde(skip_serializing, default)]
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