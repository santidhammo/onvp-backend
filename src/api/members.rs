use crate::model::setup::FirstOperator;
use crate::{dal, DbPool};
use actix_web::web::Json;
use actix_web::{get, post, web, HttpResponse, Responder};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;

pub const CONTEXT: &str = "/api/members";

#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Receive a list of members", body=[Member])
    )
)]
#[get("/list")]
pub async fn list() -> impl Responder {
    let members: Vec<crate::model::members::Member> = vec![];
    HttpResponse::Ok().json(members)
}

/// Set up the first operator
///
/// The first operator should contain enough information to create a member with the operator role,
/// and the associated details, including the address details, such that the system can be started.
/// The whole operation is performed using two steps:
/// 1. Enter the data into the database;
/// 2. Let the frontend navigate to the account activation step using a **TOTP** solution.
///
/// ⚠️ If an operator already exists, this API call (for obvious reasons) becomes invalid.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Created a new first operator", body=[String]),
        (status = 400, description = "Bad Request", body=[String]),
        (status = 500, description = "Internal backend error", body=[String])
    )
)]
#[post("/setup_first_operator")]
pub async fn setup_first_operator(
    pool: web::Data<DbPool>,
    first_operator: Json<FirstOperator>,
) -> impl Responder {
    // First check if there are already operators:
    let has_operators_result = dal::members::has_operators(&pool);
    let has_operators = match has_operators_result {
        Ok(result) => result,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let validation_string = Alphanumeric.sample_string(&mut thread_rng(), 32);

    if !has_operators {
        let result =
            dal::members::create_first_operator(&pool, &first_operator, &validation_string);
        match result {
            Ok(_) => HttpResponse::Ok().json(validation_string),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::BadRequest().body(())
    }
}
