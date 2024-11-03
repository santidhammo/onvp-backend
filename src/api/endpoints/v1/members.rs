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

//! Members are a very core component of the backend and involve a lot of interfaces regarding
//! member management as well as performing requests regarding members from normal website usage.

use crate::generic::result::{BackendError, BackendResult};
use crate::model::interface::client::UserClaims;
use crate::model::interface::commands::{
    MemberActivationCommand, MemberImageUploadCommand, MemberRegisterCommand,
    MemberUpdateAddressCommand, MemberUpdateCommand, MemberUpdatePrivacyInfoSharingCommand,
};
use crate::model::interface::responses::{
    ImageAssetIdResponse, MemberAddressResponse, MemberPrivacyInfoSharingResponse, MemberResponse,
    WorkgroupResponse,
};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::command::{
    MemberActivationCommandService, MemberCommandService, MemberPictureCommandService,
};
use crate::services::definitions::request::{MemberPictureRequestService, MemberRequestService};
use actix_web::web::{Bytes, Data, Json, Path, Query};
use actix_web::{delete, get, post, HttpResponse};
use std::ops::Deref;
use totp_rs::TOTP;

/// Register a member
///
/// Registers a new member with the necessary details. Sends an E-Mail to the
/// newly registered member to activate the account.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Successful registration", body=i32),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/")]
pub async fn register(
    controller: Data<dyn MemberCommandService>,
    command: Json<MemberRegisterCommand>,
) -> BackendResult<Json<i32>> {
    Ok(Json(controller.register_inactive(&command)?))
}

/// Search for members
///
/// Searches on first name, last name and/or email address matching the given query.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "A list of matching members", body=SearchResult<MemberResponse>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    ),
    params(
        ("q" = String, Query, description = "Part of the first name, last name and/or email address"),
        ("p" = Option<String>, Query, description = "The page offset to use (counting from 0)")
    )
)]
#[get("/search")]
pub async fn search(
    service: Data<dyn MemberRequestService>,
    search_params: Query<SearchParams>,
) -> BackendResult<Json<SearchResult<MemberResponse>>> {
    Ok(Json(service.search(search_params.deref())?))
}

/// Get a member and the primary detail by id
///
/// Searches for a member and the primary detail by using the member identifier. If found,
/// a single record with the member and primary detail is returned.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member and primary detail", body=MemberResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{id}")]
pub async fn find(
    service: Data<dyn MemberRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<MemberResponse>> {
    Ok(Json(service.find_by_id(id.into_inner())?))
}

/// Gets a member address by id
///
/// Searches for a member address by using the member identifier. If found,
/// a single record with the member address is returned.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member address", body=MemberAddressResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{id}/address")]
pub async fn find_address(
    controller: Data<dyn MemberRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<MemberAddressResponse>> {
    Ok(Json(controller.find_address_by_id(id.into_inner())?))
}

/// Gets a member privacy information sharing details
///
/// Searches for a member privacy information sharing details by using the member identifier.
/// If found, a single record with the member privacy information sharing details is returned.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member privacy information sharing details", body=MemberPrivacyInfoSharingResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{id}/privacy")]
pub async fn find_privacy_info_sharing(
    controller: Data<dyn MemberRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<MemberPrivacyInfoSharingResponse>> {
    Ok(Json(
        controller.find_privacy_info_sharing_by_id(id.into_inner())?,
    ))
}

/// Save a member and the primary detail by id
///
/// Updates an existing member and primary detail record given the data.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member is updated"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/{id}")]
pub async fn update(
    service: Data<dyn MemberCommandService>,
    id: Path<i32>,
    command: Json<MemberUpdateCommand>,
) -> BackendResult<HttpResponse> {
    service.update(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Update the address information of a member
///
/// Given the address details of a member, saves te address details
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[post("/{id}/address")]
pub async fn update_address(
    service: Data<dyn MemberCommandService>,
    id: Path<i32>,
    command: Json<MemberUpdateAddressCommand>,
) -> BackendResult<HttpResponse> {
    service.update_address(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Update the privacy information sharing details of a member
///
/// Given the new privacy information sharing details of a member, save the details
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[post("/{id}/privacy")]
pub async fn update_privacy_info_sharing(
    service: Data<dyn MemberCommandService>,
    id: Path<i32>,
    command: Json<MemberUpdatePrivacyInfoSharingCommand>,
) -> BackendResult<HttpResponse> {
    service.update_privacy_info_sharing(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Get the work groups of a member
///
/// Given the member identification, get the associated work groups
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "List of work groups is returned", body=[WorkgroupResponse]),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[get("/{id}/workgroups")]
pub async fn find_workgroups(
    service: Data<dyn MemberRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<Vec<WorkgroupResponse>>> {
    Ok(Json(service.find_workgroups(id.into_inner())?))
}

/// Upload the picture of a member
///
/// Uploads the picture of a member, adjusting it to the appropriate size by cropping it and
/// scaling it automatically. A multitude of file types are supported, but the resulting file type
/// will always be of the PNG type.
#[utoipa::path(
    request_body(content(("image/png"), ("image/jpg"))),
    tag = "members",
    responses(
        (status = 200, description = "Successful upload of the picture", body=String),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/{id}/picture.png")]
pub async fn upload_picture_asset(
    service: Data<dyn MemberPictureCommandService>,
    id: Path<i32>,
    data: Bytes,
) -> BackendResult<Json<String>> {
    let command = MemberImageUploadCommand::try_from(&data)?;
    Ok(Json(service.upload(id.into_inner(), &command)?))
}

/// Retrieves the picture of a member, if available
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Successful picture retrieval", content_type="image/png"),
        (status = 410, description = "Picture not available"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{id}/picture.png")]
pub async fn picture_asset(
    service: Data<dyn MemberPictureRequestService>,
    id: Path<i32>,
    claims: UserClaims,
) -> BackendResult<HttpResponse> {
    let result = service.find_asset_by_member_id(id.into_inner(), &claims)?;
    match result {
        None => Ok(HttpResponse::Gone().finish()),
        Some(data) => Ok(HttpResponse::Ok()
            .insert_header(data.content_type)
            .body(Bytes::from(data.bytes))),
    }
}

/// Retrieves the picture asset identifier of a member
///
/// If a member has a picture asset identifier, retrieves it. If the result is empty, no picture
/// is available.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Successful picture retrieval", body=[Option<String>]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/{id}/picture")]
pub async fn picture(
    service: Data<dyn MemberPictureRequestService>,
    id: Path<i32>,
    claims: UserClaims,
) -> BackendResult<Json<ImageAssetIdResponse>> {
    Ok(Json(
        service.find_asset_id_by_member_id(id.into_inner(), &claims)?,
    ))
}

/// Generate an activation code
///
/// Generates an activation code for a user to be activated
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "The activation code (in QR form)", body=[String]),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error")
    )
)]
#[get("/activation/code/{activation_string}")]
pub async fn activation_code(
    service: Data<dyn MemberRequestService>,
    activation_string: Path<String>,
) -> BackendResult<Json<String>> {
    let member_response = service.find_by_activation_string(&activation_string)?;
    let totp: TOTP = member_response.try_into()?;
    Ok(Json(
        totp.get_qr_base64()
            .map_err(|e| BackendError::qr_code_generation(e))?,
    ))
}

/// Activate a member
///
/// Returns if the member is activated if a member can be activated. If a member does not exist,
/// returns a Bad Request. if a member is already activated by the activation string it also returns
/// a Bad Request.
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member is activated"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/activation/activate")]
pub async fn activate(
    service: Data<dyn MemberActivationCommandService>,
    command: Json<MemberActivationCommand>,
) -> BackendResult<HttpResponse> {
    service.activate(&command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Unregister a member
///
/// Unregisters an existing member
#[utoipa::path(
    tag = "members",
    responses(
        (status = 200, description = "Member is unregistered"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[delete("/{id}")]
pub async fn unregister(
    service: Data<dyn MemberCommandService>,
    id: Path<i32>,
) -> BackendResult<HttpResponse> {
    service.unregister(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}
