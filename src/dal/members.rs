use crate::schema::member_role_associations::dsl;
use crate::security::Role;
use crate::DbPool;
use actix_web::web::Data;
use diesel::prelude::*;

pub fn has_operators(pool: Data<DbPool>) -> Result<bool, String> {
    let mut conn = pool.get().map_err(|e| e.to_string())?;

    let count: i64 = dsl::member_role_associations
        .filter(dsl::system_role.eq(Role::Operator))
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(count != 0)
}
