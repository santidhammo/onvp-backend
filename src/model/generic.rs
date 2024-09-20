//! Contains general use components which may be used throughout the system

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// To be able to search for several kinds of records, a generic search parameter structure is
/// set up to be able to perform those search requests. The "query" or "q" parameter is used to
/// set up the text to query on, the "page_offset" or "p" parameter is used to indicate which
/// page offset to use.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct SearchParams {
    /// The string to search on
    #[serde(rename = "q")]
    pub query: Option<String>,

    /// The page to query
    #[serde(rename = "p", default)]
    pub page_offset: usize,
}

/// To facilitate the results of a search operation, a generic container is used which contains
/// the search results themselves, but also the total count of found results, the page offset
/// returned and the count of pages in the system.
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult<T: Serialize> {
    pub total_count: usize,
    pub page_offset: usize,
    pub page_count: usize,
    pub rows: Vec<T>,
}
