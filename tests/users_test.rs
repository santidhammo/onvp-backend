use chrono::TimeDelta;
use onvp_backend::model::security::Role;

mod common;

#[test]
fn test_find_member_details_by_search_string() {
    let pool = common::setup();
    let mut conn = pool.get().unwrap();
    let expected_count_of_members = 20usize;
    onvp_backend::dal::mock::members::create(
        &mut conn,
        expected_count_of_members as i32,
        TimeDelta::minutes(5),
        Role::Member,
    )
    .expect("Could not create members");
    let result = onvp_backend::dal::members::find_members_with_details_by_search_string(
        &mut conn,
        &"".to_owned(),
        expected_count_of_members + 1,
        0,
    )
    .expect("Could not find members");
    assert_eq!(expected_count_of_members, result.total_count);
    assert_eq!(expected_count_of_members, result.rows.len());
}
