use crate::model::members::Member;
use crate::schema::members;
use crate::{dal, schema, Error};
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::info;

pub fn clean_late_non_activated_members(pool: dal::DbPool) -> Result<(), Error> {
    let mut conn = pool.get()?;

    conn.transaction::<_, Error, _>(|conn| {
        let activated_filter = schema::members::activated.eq(false);
        let activation_time_elapsed_filter =
            schema::members::activation_time.lt(chrono::Utc::now().naive_utc());
        let result = schema::members::table
            .select(members::all_columns)
            .filter(activated_filter.and(activation_time_elapsed_filter))
            .load::<Member>(conn)?;

        let mut deleted = 0;

        for member in result {
            {
                let details =
                    dal::members::get_member_details_by_id(conn, &member.member_details_id)?;
                info!(
                    "Deleting member: {} with name: {}",
                    member.id,
                    details.name()
                );
            }

            let result = diesel::delete(&member).execute(conn)?;
            if result != 1 {
                return Err(Error::not_enough_records());
            }
            dal::members::delete_member_details_by_id(conn, member.member_details_id)?;
            dal::members::delete_member_address_details_by_id(
                conn,
                member.member_address_details_id,
            )?;
            deleted += 1;
        }

        info!("Deleted {deleted} members");
        Ok(())
    })?;
    Ok(())
}
