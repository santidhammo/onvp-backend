/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024-2025.  Sjoerd van Leent
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
use crate::generic::lazy::OTP_CIPHER;
use crate::generic::result::{BackendError, BackendResult};
use crate::model::primitives::{EventDate, Role};
use crate::model::storage::entities::{Image, MailTemplate, MusicalInstrument, Page, Workgroup};
use crate::model::storage::extended_entities::{ExtendedMember, FacebookMember};
use actix_web::cookie::Cookie;
use actix_web::http::header::ContentType;
use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::Aead;
use base64::engine::general_purpose;
use base64::Engine;
use serde::Serialize;
use std::ops::Deref;
use totp_rs::{Algorithm, Secret, TOTP};
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

    #[schema(example = true)]
    pub activated: bool,

    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "John Doe")]
    pub full_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,

    #[serde(skip)]
    pub nonce: String,

    #[serde(skip)]
    pub activation_string: String,

    #[schema(example = "Description of this member")]
    pub description: Option<String>,
}

/// Converts an Extended Member into a Member Response used by the associated services
impl From<&ExtendedMember> for MemberResponse {
    fn from(value: &ExtendedMember) -> Self {
        Self {
            id: value.id,
            musical_instrument_id: value.musical_instrument_id,
            picture_asset_id: value.picture_asset_id.clone(),
            activated: value.activated,
            first_name: value.member_detail.first_name.clone(),
            last_name: value.member_detail.last_name.clone(),
            full_name: format!(
                "{} {}",
                value.member_detail.first_name, value.member_detail.last_name
            )
            .trim()
            .to_string(),
            email_address: value.member_detail.email_address.clone(),
            phone_number: value.member_detail.phone_number.clone(),
            nonce: value.nonce.clone(),
            activation_string: value.activation_string.clone(),
            description: value.description.clone(),
        }
    }
}

/// Attempts to generate a TOTP (one-time password)
impl TryInto<TOTP> for MemberResponse {
    type Error = BackendError;

    fn try_into(self) -> BackendResult<TOTP> {
        let nonce = self.decoded_nonce()?;
        let activation_bytes = self.activation_string.as_bytes();
        let otp_cipher = OTP_CIPHER.deref();
        let cipher_text = otp_cipher.encrypt(&nonce, activation_bytes)?;
        self.generate_totp(cipher_text)
    }
}

impl MemberResponse {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string()
    }
    fn generate_totp(&self, cipher_text: Vec<u8>) -> BackendResult<TOTP> {
        Ok(TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Raw(cipher_text).to_bytes().unwrap(),
            Some("ONVP".to_owned()),
            self.email_address.to_string(),
        )?)
    }

    fn decoded_nonce(&self) -> Result<GenericArray<u8, U12>, BackendError> {
        let decoded = general_purpose::STANDARD.decode(&self.nonce)?;

        let buffer: [u8; 12] = decoded[..].try_into().map_err(|_| {
            BackendError::insufficient_bytes("Not enough bytes available in base64 decoded Nonce")
        })?;
        GenericArray::try_from(buffer)
            .map_err(|_| BackendError::insufficient_bytes("Not enough decoded bytes in Nonce"))
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

impl From<&ExtendedMember> for MemberAddressResponse {
    fn from(value: &ExtendedMember) -> Self {
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
pub struct MemberPrivacyInfoSharingResponse {
    #[serde(default)]
    pub id: i32,

    #[schema(example = true)]
    pub allow: bool,
}

impl From<&ExtendedMember> for MemberPrivacyInfoSharingResponse {
    fn from(value: &ExtendedMember) -> Self {
        Self {
            id: value.id,
            allow: value.allow_privacy_info_sharing,
        }
    }
}

/// Used to reply on login and refresh calls to the authorization, including the member response
/// of the member logged in, and the roles of that member.
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationResponse {
    pub member: MemberResponse,
    pub composite_roles: Vec<Role>,
    #[serde(skip)]
    pub cookies: Vec<Cookie<'static>>,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkgroupResponse {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "Orchestra Committee")]
    pub name: String,
}

impl From<&Workgroup> for WorkgroupResponse {
    fn from(value: &Workgroup) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageAssetIdResponse {
    #[schema(example = "FOOBAR")]
    pub asset_id: Option<String>,
}

/// Image response containing the bytes of the image and the content type
pub struct ImageResponse {
    /// The bytes contained in the response
    pub bytes: Vec<u8>,

    /// The content-type of the response
    pub content_type: ContentType,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FacebookResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "French Horn")]
    pub musical_instrument_name: Option<String>,

    #[schema(example = "https://someurl.void")]
    pub musical_instrument_url: Option<String>,

    #[schema(example = "xyz.png")]
    pub picture_asset_id: Option<String>,

    #[schema(example = "John Doe")]
    pub full_name: String,

    #[schema(example = "Description of this member")]
    pub description: Option<String>,

    pub workgroup_names: Vec<String>,

    pub roles: Vec<Role>,
}

impl
    From<(
        &FacebookMember,
        &Option<MusicalInstrument>,
        &Vec<Workgroup>,
        &Vec<Role>,
    )> for FacebookResponse
{
    fn from(
        (facebook_member, instrument, workgroups, roles): (
            &FacebookMember,
            &Option<MusicalInstrument>,
            &Vec<Workgroup>,
            &Vec<Role>,
        ),
    ) -> Self {
        let (musical_instrument_name, musical_instrument_url): (Option<String>, Option<String>) =
            match instrument {
                None => (None, None),
                Some(instrument) => (
                    Some(instrument.name.clone()),
                    instrument.wikipedia_url.clone(),
                ),
            };

        Self {
            id: facebook_member.id,
            musical_instrument_name,
            musical_instrument_url,
            picture_asset_id: facebook_member.picture_asset_id.clone(),
            full_name: format!(
                "{} {}",
                facebook_member.first_name, facebook_member.last_name
            )
            .trim()
            .to_string(),
            description: facebook_member.description.clone(),
            workgroup_names: workgroups.iter().map(|w| w.name.clone()).collect(),
            roles: roles.clone(),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedPageResponse {
    #[schema(example = 1)]
    id: i32,

    #[schema(example = "Foo")]
    title: String,

    #[schema(example = 0)]
    order_number: i32,

    event_date: Option<EventDate>,

    parent_id: Option<i32>,

    roles: Vec<Role>,

    end_event_date: Option<EventDate>,
}

impl From<(&Page, &Vec<Role>)> for ExtendedPageResponse {
    fn from((page, roles): (&Page, &Vec<Role>)) -> Self {
        Self {
            id: page.id,
            title: page.title.clone(),
            event_date: page.event_date.map(|e| EventDate::from(&e)),
            roles: roles.clone(),
            parent_id: page.parent_id,
            order_number: page.order_number,
            end_event_date: page.end_event_date.map(|e| EventDate::from(&e)),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse {
    #[schema(example = 1)]
    id: i32,

    #[schema(example = "Foo")]
    title: String,

    #[schema(example = 0)]
    order_number: i32,

    parent_id: Option<i32>,

    event_date: Option<EventDate>,

    end_event_date: Option<EventDate>,
}

impl From<&Page> for PageResponse {
    fn from(value: &Page) -> Self {
        Self {
            id: value.id,
            parent_id: value.parent_id,
            title: value.title.clone(),
            order_number: value.order_number,
            event_date: value.event_date.map(|e| EventDate::from(&e)),
            end_event_date: value.end_event_date.map(|e| EventDate::from(&e)),
        }
    }
}

/// Image response containing extended metadata of the image
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetaDataResponse {
    /// The bytes contained in the response
    pub id: i32,

    /// The asset title of the image
    pub title: String,

    /// The asset name of the image
    pub asset: String,

    /// The roles belonging to the image
    pub roles: Vec<Role>,
}

impl From<(&Image, &Vec<Role>)> for ImageMetaDataResponse {
    fn from((image, roles): (&Image, &Vec<Role>)) -> Self {
        Self {
            id: image.id,
            title: image.title.clone(),
            asset: image.asset.clone(),
            roles: roles.clone(),
        }
    }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MusicalInstrumentResponse {
    #[schema(example = 1)]
    id: i32,

    #[schema(example = "Foo")]
    name: String,

    wikipedia_url: Option<String>,
}

impl From<&MusicalInstrument> for MusicalInstrumentResponse {
    fn from(value: &MusicalInstrument) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            wikipedia_url: value.wikipedia_url.clone(),
        }
    }
}

/// Mail template containing the body of the template
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MailTemplateResponse {
    /// The identifier of the email template
    #[schema(example = 1)]
    id: i32,

    /// The (unique) name of the email template
    #[schema(example = "Foo")]
    name: String,

    /// The body (content) of the email template
    #[schema(example = "Lorem ipsum dolor sit amet")]
    body: String,
}

impl From<&MailTemplate> for MailTemplateResponse {
    fn from(value: &MailTemplate) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            body: value.body.clone(),
        }
    }
}

/// Identification of the mail template
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MailTemplateNameResponse {
    /// The identifier of the email template
    #[schema(example = 1)]
    id: i32,

    /// The (unique) name of the email template
    #[schema(example = "Foo")]
    name: String,
}

impl From<(i32, &str)> for MailTemplateNameResponse {
    fn from((id, name): (i32, &str)) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}
