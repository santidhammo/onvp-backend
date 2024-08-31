use crate::{dal, DbPool};
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const CONTEXT: &str = "/members";

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct Member {
    #[schema(example = 1)]
    member_details_id: u32,

    #[schema(example = 1)]
    member_address_details_id: u32,

    #[schema(example = 1)]
    musical_instrument_id: u32,

    #[schema(example = "xyz.png")]
    picture_asset_id: String,

    #[schema(example = false)]
    allow_privacy_info_sharing: bool,
}

#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Receive a list of members", body=[Member])
    )
)]
#[get("/list")]
pub async fn list() -> impl Responder {
    let members = vec![Member {
        member_details_id: 1,
        member_address_details_id: 1,
        musical_instrument_id: 1,
        picture_asset_id: "hello".to_owned(),
        allow_privacy_info_sharing: true,
    }];
    HttpResponse::Ok().json(members)
}

#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Returns whether or not operators are available", body=[bool]),
        (status = 500, description = "Internal server error", body=[String])
    )
)]
#[get("/operator_check")]
pub async fn operator_check(pool: web::Data<DbPool>) -> impl Responder {
    let has_operators_result = web::block(move || dal::members::has_operators(pool)).await;
    match has_operators_result {
        Ok(result) => match result {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
