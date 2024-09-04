use crate::model::members::Member;
use crate::model::FirstOperator;
use crate::{dal, DbPool};
use actix_web::error::BlockingError;
use actix_web::web::Json;
use actix_web::{get, post, web, HttpResponse, Responder};
use diesel::Connection;
use serde::{Deserialize, Serialize};

pub const CONTEXT: &str = "/api/members";

#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Receive a list of members", body=[Member])
    )
)]
#[get("/list")]
pub async fn list() -> impl Responder {
    let members = vec![Member {
        id: None,
        member_details_id: 1,
        member_address_details_id: 1,
        musical_instrument_id: Some(1),
        picture_asset_id: Some("hello".to_owned()),
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
    let has_operators_result = dal::members::has_operators(&pool);
    match has_operators_result {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// Creates the first operator if no operators are found within the application.
///
/// The first operator should contain enough information to create a member with the operator role
/// and it's associated details and address details.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Created a new first operator"),
        (status = 400, description = "Bad Request", body=[String]),
        (status = 500, description = "Internal server error", body=[String])
    )
)]
#[post("/create_first_operator")]
pub async fn create_first_operator(
    pool: web::Data<DbPool>,
    first_operator: Json<FirstOperator>,
) -> impl Responder {
    // First check if there are already operators:
    let has_operators_result = dal::members::has_operators(&pool);
    let has_operators = match has_operators_result {
        Ok(result) => result,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    if !has_operators {
        let result = dal::members::create_first_operator(&pool, &first_operator);
        match result {
            Ok(_) => HttpResponse::Ok().body(()),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body(())
    }
}
