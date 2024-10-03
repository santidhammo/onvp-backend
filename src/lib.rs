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

pub mod api;
pub mod commands;
pub mod dal;
pub mod generic;
mod injection;
pub mod model;
mod repositories;
pub mod schema;
mod services;

use crate::generic::result::BackendResult;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use std::env::var;
use std::path::PathBuf;
use std::sync::LazyLock;

fn generate_asset_id() -> String {
    Alphanumeric.sample_string(&mut thread_rng(), 16)
}

fn path_for_asset_id(asset_id: &str) -> BackendResult<PathBuf> {
    let mut pb = PathBuf::new();
    pb.push(var("ASSETS_PATH")?);
    pb.push(asset_id);
    Ok(pb)
}

/// Returns the source code URL of the backend, this can be overwritten using the
/// BACKEND_SOURCE_CODE_URL environment variable. This is useful if for example a fork is used
pub static BACKEND_SOURCE_CODE_URL: LazyLock<String> = LazyLock::new(|| {
    var("BACKEND_SOURCE_CODE_URL")
        .unwrap_or("https://github.com/santidhammo/onvp-backend".to_owned())
});

/// Returns the source code URL of the frontend, this can be overwritten using the
/// FRONTEND_SOURCE_CODE_URL environment variable. This is useful if for example a fork is used
pub static FRONTEND_SOURCE_CODE_URL: LazyLock<String> = LazyLock::new(|| {
    var("FRONTEND_SOURCE_CODE_URL")
        .unwrap_or("https://github.com/santidhammo/onvp-frontend".to_owned())
});
