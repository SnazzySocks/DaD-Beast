//! Faceted search for filtering and aggregation

use crate::error::{SearchError, SearchResult};
use crate::schema::TorrentDocument;
use meilisearch_sdk::Index;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Facet distribution type
pub type FacetDistribution = HashMap<String, HashMap<String, usize>>;

/// Facet configuration
#[derive(Debug, Clone)]
pub struct Facet {
    /// Field name for faceting
    pub field: String,
    
    /// Maximum number of facet values to return
    pub max_values: Option<usize>,
    
    /// Optional filter to apply before computing facets
    pub filter: Option<String>,
}

impl Facet {
    /// Create a new facet
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            max_values: None,
            filter: None,
        }
    }

    /// Set maximum number of facet values
    pub fn with_max_values(mut self, max: usize) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Set filter for facet computation
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// Facet result with counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetResult {
    pub field: String,
    pub values: Vec<FacetValue>,
}

/// Individual facet value with count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: usize,
}

impl FacetResult {
    /// Sort facet values by count (descending)
    pub fn sort_by_count(&mut self) {
        self.values.sort_by(|a, b| b.count.cmp(&a.count));
    }

    /// Sort facet values alphabetically
    pub fn sort_by_value(&mut self) {
        self.values.sort_by(|a, b| a.value.cmp(&b.value));
    }

    /// Limit the number of facet values
    pub fn limit(&mut self, max: usize) {
        self.values.truncate(max);
    }
}

/// Faceted search query builder
#[derive(Debug, Clone)]
pub struct FacetedSearch {
    /// Search query
    pub query: String,
    
    /// Facets to compute
    pub facets: Vec<String>,
    
    /// Optional filter
    pub filter: Option<String>,
}

impl FacetedSearch {
    /// Create a new faceted search
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            facets: Vec::new(),
            filter: None,
        }
    }

    /// Add a facet field
    pub fn add_facet(mut self, field: impl Into<String>) -> Self {
        self.facets.push(field.into());
        self
    }

    /// Add multiple facet fields
    pub fn add_facets(mut self, fields: Vec<String>) -> Self {
        self.facets.extend(fields);
        self
    }

    /// Set filter
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Execute faceted search
    pub async fn execute(&self, index: &Index) -> SearchResult<FacetSearchResults> {
        let mut search = index.search();
        
        search.with_query(&self.query);
        
        if let Some(filter) = &self.filter {
            search.with_filter(filter);
        }
        
        if !self.facets.is_empty() {
            search.with_facets(meilisearch_sdk::Selectors::Some(&self.facets));
        }
        
        let results = search.execute::<TorrentDocument>().await?;
        
        Ok(FacetSearchResults {
            facet_distribution: results.facet_distribution.unwrap_or_default(),
            total_hits: results.estimated_total_hits.unwrap_or(0),
        })
    }
}

/// Faceted search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetSearchResults {
    pub facet_distribution: FacetDistribution,
    pub total_hits: u64,
}

impl FacetSearchResults {
    /// Get facet results for a specific field
    pub fn get_facet(&self, field: &str) -> Option<FacetResult> {
        let values = self.facet_distribution.get(field)?;
        
        let mut facet_values: Vec<FacetValue> = values
            .iter()
            .map(|(value, count)| FacetValue {
                value: value.clone(),
                count: *count,
            })
            .collect();
        
        facet_values.sort_by(|a, b| b.count.cmp(&a.count));
        
        Some(FacetResult {
            field: field.to_string(),
            values: facet_values,
        })
    }

    /// Get all facet results
    pub fn get_all_facets(&self) -> Vec<FacetResult> {
        let mut results = Vec::new();
        
        for (field, values) in &self.facet_distribution {
            let mut facet_values: Vec<FacetValue> = values
                .iter()
                .map(|(value, count)| FacetValue {
                    value: value.clone(),
                    count: *count,
                })
                .collect();
            
            facet_values.sort_by(|a, b| b.count.cmp(&a.count));
            
            results.push(FacetResult {
                field: field.clone(),
                values: facet_values,
            });
        }
        
        results.sort_by(|a, b| a.field.cmp(&b.field));
        results
    }

    /// Get category facets
    pub fn category_facets(&self) -> Option<FacetResult> {
        self.get_facet("category")
    }

    /// Get tag facets
    pub fn tag_facets(&self) -> Option<FacetResult> {
        self.get_facet("tags")
    }

    /// Get resolution facets
    pub fn resolution_facets(&self) -> Option<FacetResult> {
        self.get_facet("resolution")
    }

    /// Get year facets
    pub fn year_facets(&self) -> Option<FacetResult> {
        self.get_facet("year")
    }

    /// Get media type facets
    pub fn media_type_facets(&self) -> Option<FacetResult> {
        self.get_facet("media_type")
    }

    /// Get codec facets
    pub fn codec_facets(&self) -> Option<FacetResult> {
        self.get_facet("codec")
    }

    /// Get quality facets
    pub fn quality_facets(&self) -> Option<FacetResult> {
        self.get_facet("quality")
    }
}

/// Helper to build common faceted searches
pub struct FacetBuilder;

impl FacetBuilder {
    /// Get all available facets for browsing
    pub fn browse_facets() -> Vec<String> {
        vec![
            "category".to_string(),
            "tags".to_string(),
            "media_type".to_string(),
            "resolution".to_string(),
            "codec".to_string(),
            "quality".to_string(),
            "year".to_string(),
        ]
    }

    /// Get category-specific facets
    pub fn category_facets(category: &str) -> FacetedSearch {
        FacetedSearch::new("")
            .with_filter(format!("category = \"{}\"", category))
            .add_facets(vec![
                "tags".to_string(),
                "resolution".to_string(),
                "quality".to_string(),
                "year".to_string(),
            ])
    }

    /// Get media-type-specific facets
    pub fn media_type_facets(media_type: &str) -> FacetedSearch {
        FacetedSearch::new("")
            .with_filter(format!("media_type = \"{}\"", media_type))
            .add_facets(vec![
                "category".to_string(),
                "resolution".to_string(),
                "codec".to_string(),
                "quality".to_string(),
                "year".to_string(),
            ])
    }

    /// Get all facets without filters
    pub fn all_facets() -> FacetedSearch {
        FacetedSearch::new("").add_facets(Self::browse_facets())
    }
}

/// Dynamic facet generation based on search context
pub struct DynamicFacets;

impl DynamicFacets {
    /// Generate relevant facets based on search query
    pub fn generate_for_query(query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut facets = vec!["category".to_string()];

        // Add facets based on query content
        if query_lower.contains("movie") || query_lower.contains("film") {
            facets.extend(vec![
                "resolution".to_string(),
                "quality".to_string(),
                "codec".to_string(),
                "year".to_string(),
            ]);
        } else if query_lower.contains("tv") || query_lower.contains("series") {
            facets.extend(vec![
                "resolution".to_string(),
                "quality".to_string(),
                "year".to_string(),
            ]);
        } else if query_lower.contains("music") || query_lower.contains("album") {
            facets.extend(vec![
                "codec".to_string(),
                "quality".to_string(),
                "year".to_string(),
            ]);
        } else {
            // Default facets
            facets.extend(vec![
                "tags".to_string(),
                "media_type".to_string(),
            ]);
        }

        facets
    }

    /// Generate facets based on category
    pub fn generate_for_category(category: &str) -> Vec<String> {
        match category.to_lowercase().as_str() {
            "movies" | "tv" => vec![
                "resolution".to_string(),
                "quality".to_string(),
                "codec".to_string(),
                "year".to_string(),
                "tags".to_string(),
            ],
            "music" => vec![
                "codec".to_string(),
                "quality".to_string(),
                "year".to_string(),
                "tags".to_string(),
            ],
            "games" => vec![
                "year".to_string(),
                "tags".to_string(),
            ],
            _ => vec![
                "tags".to_string(),
                "media_type".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facet_builder() {
        let facets = FacetBuilder::browse_facets();
        assert!(!facets.is_empty());
        assert!(facets.contains(&"category".to_string()));
    }

    #[test]
    fn test_dynamic_facets_query() {
        let facets = DynamicFacets::generate_for_query("action movie");
        assert!(facets.contains(&"resolution".to_string()));
        assert!(facets.contains(&"quality".to_string()));
    }

    #[test]
    fn test_dynamic_facets_category() {
        let facets = DynamicFacets::generate_for_category("movies");
        assert!(facets.contains(&"resolution".to_string()));
        assert!(facets.contains(&"codec".to_string()));
    }

    #[test]
    fn test_facet_result_sorting() {
        let mut result = FacetResult {
            field: "test".to_string(),
            values: vec![
                FacetValue { value: "b".to_string(), count: 10 },
                FacetValue { value: "a".to_string(), count: 20 },
                FacetValue { value: "c".to_string(), count: 5 },
            ],
        };

        result.sort_by_count();
        assert_eq!(result.values[0].value, "a");
        assert_eq!(result.values[0].count, 20);

        result.sort_by_value();
        assert_eq!(result.values[0].value, "a");
    }
}
