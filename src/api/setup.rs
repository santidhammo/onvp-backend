use crate::{dal, DbPool};
use actix_web::{get, web, HttpResponse, Responder};

pub const CONTEXT: &str = "api/setup";

/// Should set up be started
///
/// Checks if the software should run the set up procedure. Returns true when there are no
/// (operator) members. In that case, the set up procedure should be started.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Returns whether or not operators are available", body=[bool]),
        (status = 500, description = "Internal backend error", body=[String])
    )
)]
#[get("/should_setup")]
pub async fn should_setup(pool: web::Data<DbPool>) -> impl Responder {
    let has_operators_result = dal::members::has_operators(&pool);
    match has_operators_result {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
