//! Search document schema and configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Torrent document structure for Meilisearch indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentDocument {
    /// Unique torrent ID (primary key)
    pub id: Uuid,
    
    /// Torrent name (highly searchable, weight: 10)
    pub name: String,
    
    /// Torrent description (searchable, weight: 3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Info hash for torrent identification
    pub info_hash: String,
    
    /// Category (filterable)
    pub category: String,
    
    /// Tags (searchable with weight: 5, filterable)
    pub tags: Vec<String>,
    
    /// Uploader username (filterable)
    pub uploader: String,
    
    /// Uploader ID
    pub uploader_id: Uuid,
    
    /// Torrent size in bytes (filterable, sortable)
    pub size: i64,
    
    /// Number of seeders (filterable, sortable)
    pub seeders: i32,
    
    /// Number of leechers (filterable, sortable)
    pub leechers: i32,
    
    /// Number of completed downloads (sortable)
    pub snatched: i32,
    
    /// Upload timestamp (filterable, sortable)
    pub uploaded_at: DateTime<Utc>,
    
    /// Media type (movie, tv, music, game, etc.) (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    
    /// Video resolution (1080p, 720p, etc.) (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    
    /// Video/audio codec (H.264, x265, FLAC, etc.) (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    
    /// Quality indicator (WEB-DL, BluRay, etc.) (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    
    /// TMDB ID for movies/TV shows (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<i32>,
    
    /// IGDB ID for games (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub igdb_id: Option<i32>,
    
    /// Release year (filterable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    
    /// Whether torrent is freeleech (filterable)
    #[serde(default)]
    pub is_freeleech: bool,
    
    /// Whether torrent has double upload (filterable)
    #[serde(default)]
    pub is_double_upload: bool,
    
    /// Featured/sticky status (filterable, sortable)
    #[serde(default)]
    pub is_featured: bool,
    
    /// Number of files in torrent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_count: Option<i32>,
    
    /// Average rating (sortable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    
    /// Number of comments (sortable)
    #[serde(default)]
    pub comment_count: i32,
}

/// Meilisearch ranking rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingRules {
    pub rules: Vec<String>,
}

impl Default for RankingRules {
    fn default() -> Self {
        Self {
            rules: vec![
                "typo".to_string(),      // Prioritize results with fewer typos
                "words".to_string(),     // Prioritize results with more query words
                "proximity".to_string(), // Prioritize results where query words are close
                "attribute".to_string(), // Prioritize results in important attributes
                "sort".to_string(),      // Apply custom sorting
                "exactness".to_string(), // Prioritize exact matches
            ],
        }
    }
}

/// Meilisearch index configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Index name
    pub index_name: String,
    
    /// Primary key field
    pub primary_key: String,
    
    /// Searchable attributes with weights
    pub searchable_attributes: Vec<String>,
    
    /// Filterable attributes
    pub filterable_attributes: Vec<String>,
    
    /// Sortable attributes
    pub sortable_attributes: Vec<String>,
    
    /// Ranking rules
    pub ranking_rules: RankingRules,
    
    /// Displayed attributes (empty = all)
    pub displayed_attributes: Vec<String>,
    
    /// Stop words to ignore in search
    pub stop_words: Vec<String>,
    
    /// Synonyms for search
    pub synonyms: std::collections::HashMap<String, Vec<String>>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        let mut synonyms = std::collections::HashMap::new();
        
        // Add common synonyms
        synonyms.insert("movie".to_string(), vec!["film".to_string(), "cinema".to_string()]);
        synonyms.insert("tv".to_string(), vec!["television".to_string(), "series".to_string(), "show".to_string()]);
        synonyms.insert("music".to_string(), vec!["audio".to_string(), "song".to_string(), "album".to_string()]);
        synonyms.insert("game".to_string(), vec!["gaming".to_string(), "videogame".to_string()]);
        
        Self {
            index_name: "torrents".to_string(),
            primary_key: "id".to_string(),
            searchable_attributes: vec![
                "name".to_string(),        // Weight: 10 (most important)
                "tags".to_string(),        // Weight: 5
                "description".to_string(), // Weight: 3
            ],
            filterable_attributes: vec![
                "category".to_string(),
                "tags".to_string(),
                "media_type".to_string(),
                "resolution".to_string(),
                "codec".to_string(),
                "quality".to_string(),
                "uploaded_at".to_string(),
                "size".to_string(),
                "seeders".to_string(),
                "leechers".to_string(),
                "uploader".to_string(),
                "uploader_id".to_string(),
                "tmdb_id".to_string(),
                "igdb_id".to_string(),
                "year".to_string(),
                "is_freeleech".to_string(),
                "is_double_upload".to_string(),
                "is_featured".to_string(),
            ],
            sortable_attributes: vec![
                "uploaded_at".to_string(),
                "size".to_string(),
                "seeders".to_string(),
                "leechers".to_string(),
                "snatched".to_string(),
                "rating".to_string(),
                "comment_count".to_string(),
                "is_featured".to_string(),
            ],
            ranking_rules: RankingRules::default(),
            displayed_attributes: vec![], // Return all attributes
            stop_words: vec![
                "the".to_string(),
                "a".to_string(),
                "an".to_string(),
                "and".to_string(),
                "or".to_string(),
                "but".to_string(),
                "in".to_string(),
                "on".to_string(),
                "at".to_string(),
                "to".to_string(),
                "for".to_string(),
            ],
            synonyms,
        }
    }
}

impl SearchConfig {
    /// Create a new search configuration with custom index name
    pub fn new(index_name: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            ..Default::default()
        }
    }
    
    /// Add a searchable attribute
    pub fn add_searchable_attribute(mut self, attr: impl Into<String>) -> Self {
        self.searchable_attributes.push(attr.into());
        self
    }
    
    /// Add a filterable attribute
    pub fn add_filterable_attribute(mut self, attr: impl Into<String>) -> Self {
        self.filterable_attributes.push(attr.into());
        self
    }
    
    /// Add a sortable attribute
    pub fn add_sortable_attribute(mut self, attr: impl Into<String>) -> Self {
        self.sortable_attributes.push(attr.into());
        self
    }
    
    /// Add a synonym mapping
    pub fn add_synonym(mut self, word: impl Into<String>, synonyms: Vec<String>) -> Self {
        self.synonyms.insert(word.into(), synonyms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SearchConfig::default();
        assert_eq!(config.index_name, "torrents");
        assert_eq!(config.primary_key, "id");
        assert!(!config.searchable_attributes.is_empty());
        assert!(!config.filterable_attributes.is_empty());
        assert!(!config.sortable_attributes.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let config = SearchConfig::new("custom_index")
            .add_searchable_attribute("custom_field")
            .add_filterable_attribute("filter_field")
            .add_sortable_attribute("sort_field");
        
        assert_eq!(config.index_name, "custom_index");
        assert!(config.searchable_attributes.contains(&"custom_field".to_string()));
    }
}
