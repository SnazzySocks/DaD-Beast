//! Search query builder and execution

use crate::error::{SearchError, SearchResult};
use crate::filters::{Filter, FilterBuilder};
use crate::schema::TorrentDocument;
use meilisearch_sdk::Index;
use serde::{Deserialize, Serialize};

/// Search query builder
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Search term
    pub query: String,
    
    /// Filter string
    pub filter: Option<String>,
    
    /// Sort criteria
    pub sort: Vec<String>,
    
    /// Limit (page size)
    pub limit: usize,
    
    /// Offset (pagination)
    pub offset: usize,
    
    /// Attributes to retrieve
    pub attributes_to_retrieve: Option<Vec<String>>,
    
    /// Attributes to highlight
    pub attributes_to_highlight: Option<Vec<String>>,
    
    /// Highlight tags
    pub highlight_pre_tag: String,
    pub highlight_post_tag: String,
    
    /// Facets to compute
    pub facets: Option<Vec<String>>,
    
    /// Matching strategy
    pub matching_strategy: MatchingStrategy,
}

/// Matching strategy for search queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchingStrategy {
    /// Return documents matching all query terms (default)
    All,
    /// Return documents matching at least one query term
    Last,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            filter: None,
            sort: Vec::new(),
            limit: 20,
            offset: 0,
            attributes_to_retrieve: None,
            attributes_to_highlight: Some(vec!["name".to_string(), "description".to_string()]),
            highlight_pre_tag: "<em>".to_string(),
            highlight_post_tag: "</em>".to_string(),
            facets: None,
            matching_strategy: MatchingStrategy::All,
        }
    }
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set the search term
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Add a filter
    pub fn with_filter<F: Filter>(mut self, filter: F) -> Self {
        let filter_str = filter.to_filter_string();
        if !filter_str.is_empty() {
            self.filter = Some(match self.filter {
                Some(existing) => format!("{} AND {}", existing, filter_str),
                None => filter_str,
            });
        }
        self
    }

    /// Set filters from a filter builder
    pub fn with_filters(mut self, builder: FilterBuilder) -> Self {
        let filter_str = builder.build();
        if !filter_str.is_empty() {
            self.filter = Some(filter_str);
        }
        self
    }

    /// Set raw filter string
    pub fn with_raw_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Add a sort criterion
    pub fn with_sort(mut self, sort: SortBy) -> Self {
        self.sort.push(sort.to_string());
        self
    }

    /// Set multiple sort criteria
    pub fn with_sorts(mut self, sorts: Vec<SortBy>) -> Self {
        self.sort = sorts.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the limit (page size)
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the offset (for pagination)
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set the page number (convenience method)
    pub fn with_page(mut self, page: usize) -> Self {
        self.offset = page.saturating_sub(1) * self.limit;
        self
    }

    /// Set attributes to retrieve
    pub fn with_attributes_to_retrieve(mut self, attributes: Vec<String>) -> Self {
        self.attributes_to_retrieve = Some(attributes);
        self
    }

    /// Set attributes to highlight
    pub fn with_attributes_to_highlight(mut self, attributes: Vec<String>) -> Self {
        self.attributes_to_highlight = Some(attributes);
        self
    }

    /// Enable facets
    pub fn with_facets(mut self, facets: Vec<String>) -> Self {
        self.facets = Some(facets);
        self
    }

    /// Set matching strategy
    pub fn with_matching_strategy(mut self, strategy: MatchingStrategy) -> Self {
        self.matching_strategy = strategy;
        self
    }

    /// Quick filter by category
    pub fn with_category(self, category: impl Into<String>) -> Self {
        self.with_raw_filter(format!("category = \"{}\"", category.into()))
    }

    /// Quick filter by media type
    pub fn with_media_type(self, media_type: impl Into<String>) -> Self {
        self.with_raw_filter(format!("media_type = \"{}\"", media_type.into()))
    }

    /// Quick filter for freeleech torrents
    pub fn freeleech_only(self) -> Self {
        self.with_raw_filter("is_freeleech = true")
    }

    /// Execute the search query
    pub async fn execute(self, index: &Index) -> SearchResult<SearchResults> {
        let mut search = index.search();
        
        // Set query
        search.with_query(&self.query);
        
        // Set filter
        if let Some(filter) = &self.filter {
            search.with_filter(filter);
        }
        
        // Set sort
        if !self.sort.is_empty() {
            search.with_sort(&self.sort);
        }
        
        // Set pagination
        search.with_limit(self.limit);
        search.with_offset(self.offset);
        
        // Set attributes to retrieve
        if let Some(attributes) = &self.attributes_to_retrieve {
            search.with_attributes_to_retrieve(meilisearch_sdk::Selectors::Some(attributes));
        }
        
        // Set highlighting
        if let Some(attributes) = &self.attributes_to_highlight {
            search.with_attributes_to_highlight(meilisearch_sdk::Selectors::Some(attributes));
            search.with_highlight_pre_tag(&self.highlight_pre_tag);
            search.with_highlight_post_tag(&self.highlight_post_tag);
        }
        
        // Set facets
        if let Some(facets) = &self.facets {
            search.with_facets(meilisearch_sdk::Selectors::Some(facets));
        }
        
        // Execute search
        let results = search.execute::<TorrentDocument>().await?;
        
        Ok(SearchResults {
            hits: results.hits,
            query: self.query,
            processing_time_ms: results.processing_time_ms,
            limit: self.limit,
            offset: self.offset,
            estimated_total_hits: results.estimated_total_hits,
            facet_distribution: results.facet_distribution,
        })
    }
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Search hits with relevance ranking
    pub hits: Vec<SearchHit>,
    
    /// Original query
    pub query: String,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    
    /// Limit used
    pub limit: usize,
    
    /// Offset used
    pub offset: usize,
    
    /// Estimated total number of hits
    pub estimated_total_hits: Option<u64>,
    
    /// Facet distribution (if requested)
    pub facet_distribution: Option<std::collections::HashMap<String, std::collections::HashMap<String, usize>>>,
}

/// Single search hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// The matching document
    #[serde(flatten)]
    pub result: TorrentDocument,
    
    /// Formatted fields with highlighting (if requested)
    #[serde(rename = "_formatted")]
    pub formatted: Option<serde_json::Value>,
}

impl SearchResults {
    /// Get the current page number
    pub fn page(&self) -> usize {
        if self.limit == 0 {
            1
        } else {
            (self.offset / self.limit) + 1
        }
    }

    /// Get the total number of pages
    pub fn total_pages(&self) -> usize {
        if let Some(total) = self.estimated_total_hits {
            if self.limit == 0 {
                1
            } else {
                ((total as usize + self.limit - 1) / self.limit).max(1)
            }
        } else {
            1
        }
    }

    /// Check if there are more results
    pub fn has_next_page(&self) -> bool {
        if let Some(total) = self.estimated_total_hits {
            self.offset + self.limit < total as usize
        } else {
            false
        }
    }

    /// Check if there are previous results
    pub fn has_prev_page(&self) -> bool {
        self.offset > 0
    }

    /// Get facet counts for a specific field
    pub fn get_facet_counts(&self, field: &str) -> Option<&std::collections::HashMap<String, usize>> {
        self.facet_distribution.as_ref()?.get(field)
    }
}

/// Sort options for search results
#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    /// Newest first
    NewestFirst,
    /// Oldest first
    OldestFirst,
    /// Largest first
    LargestFirst,
    /// Smallest first
    SmallestFirst,
    /// Most seeders first
    MostSeeders,
    /// Most leechers first
    MostLeechers,
    /// Most snatches first
    MostSnatches,
    /// Highest rated first
    HighestRated,
    /// Most commented first
    MostCommented,
    /// Featured first
    FeaturedFirst,
}

impl std::fmt::Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SortBy::NewestFirst => "uploaded_at:desc",
            SortBy::OldestFirst => "uploaded_at:asc",
            SortBy::LargestFirst => "size:desc",
            SortBy::SmallestFirst => "size:asc",
            SortBy::MostSeeders => "seeders:desc",
            SortBy::MostLeechers => "leechers:desc",
            SortBy::MostSnatches => "snatched:desc",
            SortBy::HighestRated => "rating:desc",
            SortBy::MostCommented => "comment_count:desc",
            SortBy::FeaturedFirst => "is_featured:desc",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("ubuntu")
            .with_category("linux")
            .with_limit(50)
            .with_page(2);

        assert_eq!(query.query, "ubuntu");
        assert_eq!(query.limit, 50);
        assert_eq!(query.offset, 50);
    }

    #[test]
    fn test_sort_by_display() {
        assert_eq!(SortBy::NewestFirst.to_string(), "uploaded_at:desc");
        assert_eq!(SortBy::MostSeeders.to_string(), "seeders:desc");
    }

    #[test]
    fn test_search_results_pagination() {
        let results = SearchResults {
            hits: vec![],
            query: "test".to_string(),
            processing_time_ms: 10,
            limit: 20,
            offset: 40,
            estimated_total_hits: Some(100),
            facet_distribution: None,
        };

        assert_eq!(results.page(), 3);
        assert_eq!(results.total_pages(), 5);
        assert!(results.has_next_page());
        assert!(results.has_prev_page());
    }
}
