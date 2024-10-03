use crate::dal;
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::security::create_activation_string;
use crate::model::interface::prelude::*;
use crate::repositories::traits::MemberRoleRepository;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web};

pub const CONTEXT: &str = "/api/setup";

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
pub async fn should_setup(pool: web::Data<dal::DbPool>) -> BackendResult<Json<bool>> {
    let mut conn = dal::connect(&pool)?;
    dal::members::has_operators(&mut conn).map(|v| Json(!v))
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
    pool: Data<dal::DbPool>,
    first_operator: Json<FirstOperatorRegisterCommand>,
    member_role_repository: Data<dyn MemberRoleRepository>,
) -> BackendResult<Json<String>> {
    let mut conn = dal::connect(&pool)?;
    // First check if there are already operators:
    let has_operators = dal::members::has_operators(&mut conn)?;
    let activation_string = create_activation_string();

    if !has_operators {
        dal::members::register_first_operator(
            &mut conn,
            &first_operator,
            &activation_string,
            member_role_repository.get_ref(),
        )?;
        Ok(Json(activation_string))
    } else {
        Err(BackendError::bad())
    }
}
