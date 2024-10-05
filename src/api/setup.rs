use crate::generic::result::BackendResult;

use crate::model::interface::commands::FirstOperatorRegisterCommand;
use crate::services::traits::command::SetupCommandService;
use crate::services::traits::request::SetupRequestService;
use actix_web::web::{Data, Json};
use actix_web::{get, post};

pub const CONTEXT: &str = "/api/setup";

/// Should set up be started
///
/// Checks if the software should run the set up procedure. Returns true when there are no
/// (operator) members. In that case, the set up procedure should be started.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Returns whether or not operators are available", body=bool),
        (status = 500, description = "Internal backend error", body=[String])
    )
)]
#[get("/should_setup")]
pub async fn should_setup(service: Data<dyn SetupRequestService>) -> BackendResult<Json<bool>> {
    Ok(Json(service.should_setup()?))
}

/// Set up the first operator
///
/// The first operator should contain enough information to create a member with the operator role,
/// and the associated details, including the address details, such that the system can be started.
/// The whole operation is performed using two steps:
/// 1. Enter the data into the storage;
/// 2. Let the frontend navigate to the account activation step using a **TOTP** solution.
///
/// ⚠️ If an operator already exists, this API call (for obvious reasons) becomes invalid.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Created a new first operator", body=String),
        (status = 400, description = "Bad Request", body=String),
        (status = 500, description = "Internal backend error", body=String)
    )
)]
#[post("/setup_first_operator")]
pub async fn setup_first_operator(
    command: Json<FirstOperatorRegisterCommand>,
    service: Data<dyn SetupCommandService>,
) -> BackendResult<Json<String>> {
    Ok(Json(service.register_first_operator(&command)?))
}
