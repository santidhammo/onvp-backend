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

pub(crate) fn create_like_string<T: ToString>(search_string: T) -> String {
    let search_string = search_string.to_string();
    let search_string = if !search_string.starts_with("%") {
        format!("%{search_string}")
    } else {
        search_string.clone()
    };

    let search_string = if !search_string.ends_with("%") {
        format!("{search_string}%")
    } else {
        search_string.clone()
    };
    search_string
}

/// The total count of pages is the total_count divided by page_size, and if the
/// rest is > 0, one more.
pub(crate) fn calculate_page_count(page_size: usize, total_count: usize) -> usize {
    let page_count = (total_count / page_size) + if (total_count % page_size) != 0 { 1 } else { 0 };
    page_count
}
