//! # Search Crate
//!
//! This crate provides Meilisearch integration for the tracker platform.
//! It enables advanced search capabilities with faceted filtering, autocomplete,
//! and search analytics.
//!
//! ## Features
//!
//! - Full-text search with relevance ranking
//! - Faceted filtering (category, tags, media type, etc.)
//! - Advanced filtering (date range, size range, seeder count, etc.)
//! - Autocomplete suggestions
//! - Search analytics and tracking
//! - Batch indexing for performance
//! - Background indexing jobs
//!
//! ## Usage
//!
//! ```no_run
//! use search::{SearchClient, SearchQuery};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = SearchClient::new("http://localhost:7700", "master_key").await?;
//!     
//!     let query = SearchQuery::new("ubuntu")
//!         .with_category("linux")
//!         .with_limit(20);
//!     
//!     let results = client.search(query).await?;
//!     println!("Found {} results", results.hits.len());
//!     
//!     Ok(())
//! }
//! ```

pub mod analytics;
pub mod client;
pub mod facets;
pub mod filters;
pub mod indexer;
pub mod query;
pub mod schema;
pub mod suggest;

mod error;

pub use client::SearchClient;
pub use error::{SearchError, SearchResult};
pub use facets::{Facet, FacetDistribution};
pub use filters::{
    CategoryFilter, DateRangeFilter, Filter, SeedsFilter, SizeRangeFilter, TagFilter,
    UploaderFilter,
};
pub use indexer::{IndexOperation, SearchIndexer};
pub use query::{SearchQuery, SearchResults, SortBy};
pub use schema::{RankingRules, SearchConfig, TorrentDocument};
pub use suggest::{SearchSuggestion, SuggestionType};

/// Initialize the search service with default configuration
pub async fn init(
    meilisearch_host: &str,
    api_key: &str,
) -> SearchResult<SearchClient> {
    SearchClient::new(meilisearch_host, api_key).await
}
