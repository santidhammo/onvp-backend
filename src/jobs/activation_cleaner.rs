use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use dotenv::dotenv;
use log::info;
use onvp_backend::dal::get_connection;
use onvp_backend::model::members::Member;
use onvp_backend::schema::members;
use onvp_backend::{dal, initialize_db_pool};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv().ok();
    let pool = initialize_db_pool();
    let mut conn = get_connection(&pool)?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let activated_filter = members::dsl::activated.eq(false);
        let activation_time_elapsed_filter =
            members::dsl::activation_time.lt(chrono::Utc::now().naive_utc());
        let result = members::dsl::members
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
