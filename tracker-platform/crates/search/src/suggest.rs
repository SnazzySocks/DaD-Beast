//! Autocomplete and search suggestions

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use crate::schema::TorrentDocument;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Type of suggestion
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// Direct match from torrent names
    TorrentName,
    /// Tag suggestion
    Tag,
    /// Category suggestion
    Category,
    /// Recent search query
    RecentSearch,
    /// Popular search query
    PopularSearch,
    /// Uploader name
    Uploader,
}

/// Search suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    /// Suggestion text
    pub text: String,
    
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    
    /// Optional score/ranking
    pub score: f64,
    
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl SearchSuggestion {
    pub fn new(text: impl Into<String>, suggestion_type: SuggestionType) -> Self {
        Self {
            text: text.into(),
            suggestion_type,
            score: 0.0,
            metadata: None,
        }
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Autocomplete service
pub struct AutocompleteService {
    client: SearchClient,
    db: PgPool,
}

impl AutocompleteService {
    /// Create a new autocomplete service
    pub fn new(client: SearchClient, db: PgPool) -> Self {
        Self { client, db }
    }

    /// Get autocomplete suggestions for a query
    pub async fn suggest(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let mut suggestions = Vec::new();

        // Get suggestions from different sources in parallel
        let (torrent_suggestions, tag_suggestions, popular_suggestions) = tokio::join!(
            self.suggest_from_torrents(query, limit),
            self.suggest_from_tags(query, limit),
            self.get_popular_searches(limit)
        );

        // Add torrent name suggestions
        if let Ok(mut sug) = torrent_suggestions {
            suggestions.append(&mut sug);
        }

        // Add tag suggestions
        if let Ok(mut sug) = tag_suggestions {
            suggestions.append(&mut sug);
        }

        // Add popular searches if query is very short
        if query.len() <= 2 {
            if let Ok(mut sug) = popular_suggestions {
                suggestions.append(&mut sug);
            }
        }

        // Sort by score and limit
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit);

        Ok(suggestions)
    }

    /// Get suggestions from torrent names
    async fn suggest_from_torrents(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let index = self.client.index();
        
        let search_results = index
            .search()
            .with_query(query)
            .with_limit(limit)
            .with_attributes_to_retrieve(meilisearch_sdk::Selectors::Some(&["name", "seeders"]))
            .execute::<TorrentDocument>()
            .await?;

        let suggestions = search_results
            .hits
            .into_iter()
            .map(|hit| {
                let score = hit.result.seeders as f64 / 100.0; // Use seeders as relevance score
                SearchSuggestion::new(hit.result.name, SuggestionType::TorrentName)
                    .with_score(score)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get suggestions from tags
    async fn suggest_from_tags(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            SELECT tag, COUNT(*) as count
            FROM torrent_tags
            WHERE tag ILIKE $1
            GROUP BY tag
            ORDER BY count DESC
            LIMIT $2
            "#
        )
        .bind(format!("{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let tag: String = row.get("tag");
                let count: i64 = row.get("count");
                SearchSuggestion::new(tag, SuggestionType::Tag)
                    .with_score(count as f64)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get recent searches for a user
    pub async fn get_recent_searches(&self, user_id: Uuid, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT query, created_at
            FROM search_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let query: String = row.get("query");
                SearchSuggestion::new(query, SuggestionType::RecentSearch)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get popular searches across all users
    pub async fn get_popular_searches(&self, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            SELECT query, COUNT(*) as count
            FROM search_history
            WHERE created_at > NOW() - INTERVAL '7 days'
            GROUP BY query
            ORDER BY count DESC
            LIMIT $1
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let query: String = row.get("query");
                let count: i64 = row.get("count");
                SearchSuggestion::new(query, SuggestionType::PopularSearch)
                    .with_score(count as f64)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get category suggestions
    pub async fn suggest_categories(&self, query: &str) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            SELECT name, torrent_count
            FROM categories
            WHERE name ILIKE $1
            ORDER BY torrent_count DESC
            "#
        )
        .bind(format!("{}%", query))
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let name: String = row.get("name");
                let count: i64 = row.get("torrent_count");
                SearchSuggestion::new(name, SuggestionType::Category)
                    .with_score(count as f64)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get uploader suggestions
    pub async fn suggest_uploaders(&self, query: &str, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            SELECT username, upload_count
            FROM users
            WHERE username ILIKE $1
            ORDER BY upload_count DESC
            LIMIT $2
            "#
        )
        .bind(format!("{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let username: String = row.get("username");
                let count: i64 = row.get("upload_count");
                SearchSuggestion::new(username, SuggestionType::Uploader)
                    .with_score(count as f64)
            })
            .collect();

        Ok(suggestions)
    }

    /// Get category-specific suggestions
    pub async fn suggest_in_category(
        &self,
        query: &str,
        category: &str,
        limit: usize,
    ) -> SearchResult<Vec<SearchSuggestion>> {
        let index = self.client.index();
        
        let search_results = index
            .search()
            .with_query(query)
            .with_filter(&format!("category = \"{}\"", category))
            .with_limit(limit)
            .with_attributes_to_retrieve(meilisearch_sdk::Selectors::Some(&["name", "seeders"]))
            .execute::<TorrentDocument>()
            .await?;

        let suggestions = search_results
            .hits
            .into_iter()
            .map(|hit| {
                let score = hit.result.seeders as f64 / 100.0;
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), category.to_string());
                
                SearchSuggestion::new(hit.result.name, SuggestionType::TorrentName)
                    .with_score(score)
                    .with_metadata(metadata)
            })
            .collect();

        Ok(suggestions)
    }

    /// Record a search query for analytics and future suggestions
    pub async fn record_search(
        &self,
        user_id: Option<Uuid>,
        query: &str,
        results_count: u64,
    ) -> SearchResult<()> {
        sqlx::query(
            r#"
            INSERT INTO search_history (user_id, query, results_count, created_at)
            VALUES ($1, $2, $3, NOW())
            "#
        )
        .bind(user_id)
        .bind(query)
        .bind(results_count as i64)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get trending searches (searches with increasing frequency)
    pub async fn get_trending_searches(&self, limit: usize) -> SearchResult<Vec<SearchSuggestion>> {
        let rows = sqlx::query(
            r#"
            WITH recent_counts AS (
                SELECT 
                    query,
                    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as recent_count,
                    COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') as week_count
                FROM search_history
                WHERE created_at > NOW() - INTERVAL '7 days'
                GROUP BY query
                HAVING COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') > 5
            )
            SELECT 
                query,
                recent_count,
                (recent_count::float / NULLIF(week_count, 0)::float) as trend_score
            FROM recent_counts
            ORDER BY trend_score DESC
            LIMIT $1
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let suggestions = rows
            .into_iter()
            .map(|row| {
                let query: String = row.get("query");
                let trend_score: Option<f64> = row.get("trend_score");
                SearchSuggestion::new(query, SuggestionType::PopularSearch)
                    .with_score(trend_score.unwrap_or(0.0))
            })
            .collect();

        Ok(suggestions)
    }
}

/// Search-as-you-type handler
pub struct SearchAsYouType {
    service: AutocompleteService,
}

impl SearchAsYouType {
    pub fn new(service: AutocompleteService) -> Self {
        Self { service }
    }

    /// Get instant suggestions with minimum query length
    pub async fn instant_suggest(
        &self,
        query: &str,
        min_length: usize,
        limit: usize,
    ) -> SearchResult<Vec<SearchSuggestion>> {
        if query.len() < min_length {
            return Ok(Vec::new());
        }

        self.service.suggest(query, limit).await
    }

    /// Get suggestions with debouncing simulation
    /// (In real implementation, debouncing would be handled client-side)
    pub async fn debounced_suggest(
        &self,
        query: &str,
        limit: usize,
    ) -> SearchResult<Vec<SearchSuggestion>> {
        // Minimum 2 characters for suggestions
        if query.len() < 2 {
            return Ok(Vec::new());
        }

        self.service.suggest(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let suggestion = SearchSuggestion::new("test query", SuggestionType::TorrentName)
            .with_score(10.0);

        assert_eq!(suggestion.text, "test query");
        assert_eq!(suggestion.suggestion_type, SuggestionType::TorrentName);
        assert_eq!(suggestion.score, 10.0);
    }

    #[test]
    fn test_suggestion_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "movies".to_string());

        let suggestion = SearchSuggestion::new("test", SuggestionType::TorrentName)
            .with_metadata(metadata);

        assert!(suggestion.metadata.is_some());
    }
}
