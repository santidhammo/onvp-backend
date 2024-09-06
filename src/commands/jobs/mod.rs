use crate::dal::get_connection;
use crate::model::members::Member;
use crate::schema::members;
use crate::{dal, schema, DbPool};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::info;
use std::error::Error;

pub fn clean_late_non_activated_members(pool: &DbPool) -> Result<(), Box<dyn Error>> {
    let mut conn = get_connection(&pool)?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let activated_filter = schema::members::activated.eq(false);
        let activation_time_elapsed_filter =
            schema::members::activation_time.lt(chrono::Utc::now().naive_utc());
        let result = schema::members::table
            .select(members::all_columns)
            .filter(activated_filter.and(activation_time_elapsed_filter))
            .load::<Member>(conn)?;

        let mut deleted = 0;

        let first_error = result
            .iter()
            .map(|member| {
                info!("Deleting member: {}", member.id);

                let result = diesel::delete(member).execute(conn);
                if let Ok(_) = result {
                    deleted += 1
                };
                dal::members::delete_member_details_by_id(conn, member.member_details_id)?;
                dal::members::delete_member_address_details_by_id(
                    conn,
                    member.member_address_details_id,
                )?;

                result
            })
            .filter(|r| r.is_err())
            .map(|r| r.unwrap_err())
            .nth(0);

        match first_error {
            None => {
                info!("Deleted: {deleted} members");
                Ok(())
            }
            Some(e) => Err(e),
        }
    })?;
    Ok(())
}
