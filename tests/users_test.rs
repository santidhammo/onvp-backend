/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
    let result = onvp_backend::dal::members::find_with_details_by_search_string(
        &mut conn,
        &"".to_owned(),
        expected_count_of_members + 1,
        0,
    )
    .expect("Could not find members");
    assert_eq!(expected_count_of_members, result.total_count);
    assert_eq!(expected_count_of_members, result.rows.len());
}
