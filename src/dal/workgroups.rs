use crate::dal::DbConnection;
use crate::model::security::Role;
use crate::model::workgroups::RegisterCommand;
use crate::schema::*;
use crate::{Error, Result};
use diesel::prelude::*;

pub(crate) fn register(conn: &mut DbConnection, command: &RegisterCommand) -> Result<()> {
    diesel::insert_into(workgroups::table)
        .values(command)
        .execute(conn)?;
    Ok(())
}

pub(crate) fn assoc_role(conn: &mut DbConnection, workgroup_id: &i32, role: &Role) -> Result<()> {
    diesel::insert_into(workgroup_role_associations::table)
        .values((
            workgroup_role_associations::workgroup_id.eq(workgroup_id),
            workgroup_role_associations::system_role.eq(role),
        ))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn dissoc_role(conn: &mut DbConnection, workgroup_id: &i32, role: &Role) -> Result<()> {
    let deleted_rows = diesel::delete(workgroup_role_associations::table)
        .filter(
            workgroup_role_associations::workgroup_id
                .eq(workgroup_id)
                .and(workgroup_role_associations::system_role.eq(role)),
        )
        .execute(conn)?;
    if deleted_rows > 0 {
        Ok(())
    } else {
        Err(Error::not_enough_records())
    }
}
