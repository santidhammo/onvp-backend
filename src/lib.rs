pub mod api;
pub mod commands;
pub mod dal;
mod generic;
pub mod model;
pub mod result;
pub mod schema;

use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
pub use result::*;
use std::env::var;
use std::path::PathBuf;

fn generate_asset_id() -> String {
    Alphanumeric.sample_string(&mut thread_rng(), 16)
}

fn path_for_asset_id(asset_id: &str) -> Result<PathBuf> {
    let mut pb = PathBuf::new();
    pb.push(var("ASSETS_PATH")?);
    pb.push(asset_id);
    Ok(pb)
}
