//! Contains general use components which may be used throughout the system

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// This structure is to be used for search purposes, and should be deserialized from the web
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct SearchParams {
    /// The string to search on
    #[serde(rename = "q")]
    pub query: Option<String>,

    /// The page to query
    #[serde(rename = "p", default)]
    pub page_offset: usize,
}

/// This structure is used for the results of a search operation, containing a collection of
/// rows of the found results.
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult<T: Serialize> {
    pub total_count: usize,
    pub page_offset: usize,
    pub page_count: usize,
    pub rows: Vec<T>,
}
