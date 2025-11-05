//! Meilisearch client setup and management

use crate::error::{SearchError, SearchResult};
use crate::query::{SearchQuery, SearchResults};
use crate::schema::{SearchConfig, TorrentDocument};
use meilisearch_sdk::{Client, Index};
use std::sync::Arc;
use tracing::{info, warn};

/// Meilisearch client wrapper for torrent search
#[derive(Clone)]
pub struct SearchClient {
    client: Arc<Client>,
    config: SearchConfig,
}

impl SearchClient {
    /// Create a new search client
    ///
    /// # Arguments
    ///
    /// * `host` - Meilisearch server URL (e.g., "http://localhost:7700")
    /// * `api_key` - Meilisearch API key (master key or search key)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use search::SearchClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let client = SearchClient::new("http://localhost:7700", "master_key").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(host: impl AsRef<str>, api_key: impl AsRef<str>) -> SearchResult<Self> {
        let client = Client::new(host.as_ref(), Some(api_key.as_ref()));
        
        // Check connection health
        if let Err(e) = client.health().await {
            return Err(SearchError::Configuration(format!(
                "Failed to connect to Meilisearch: {}",
                e
            )));
        }
        
        info!("Connected to Meilisearch at {}", host.as_ref());
        
        let config = SearchConfig::default();
        let search_client = Self {
            client: Arc::new(client),
            config,
        };
        
        // Initialize index with configuration
        search_client.initialize_index().await?;
        
        Ok(search_client)
    }

    /// Create a new search client with custom configuration
    pub async fn with_config(
        host: impl AsRef<str>,
        api_key: impl AsRef<str>,
        config: SearchConfig,
    ) -> SearchResult<Self> {
        let client = Client::new(host.as_ref(), Some(api_key.as_ref()));
        
        // Check connection health
        if let Err(e) = client.health().await {
            return Err(SearchError::Configuration(format!(
                "Failed to connect to Meilisearch: {}",
                e
            )));
        }
        
        info!("Connected to Meilisearch at {} with custom config", host.as_ref());
        
        let search_client = Self {
            client: Arc::new(client),
            config,
        };
        
        // Initialize index with configuration
        search_client.initialize_index().await?;
        
        Ok(search_client)
    }

    /// Get the torrents index
    pub fn index(&self) -> Index {
        self.client.index(&self.config.index_name)
    }

    /// Check Meilisearch server health
    pub async fn health_check(&self) -> SearchResult<bool> {
        match self.client.health().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Initialize or update the search index with configuration
    pub async fn initialize_index(&self) -> SearchResult<()> {
        info!("Initializing search index: {}", self.config.index_name);
        
        // Create or get index
        let index = self
            .client
            .create_index(&self.config.index_name, Some(&self.config.primary_key))
            .await
            .or_else(|_| async {
                // Index might already exist, try to get it
                Ok(self.client.index(&self.config.index_name))
            })
            .await?;

        // Configure searchable attributes
        if !self.config.searchable_attributes.is_empty() {
            index
                .set_searchable_attributes(&self.config.searchable_attributes)
                .await?;
            info!("Set searchable attributes: {:?}", self.config.searchable_attributes);
        }

        // Configure filterable attributes
        if !self.config.filterable_attributes.is_empty() {
            index
                .set_filterable_attributes(&self.config.filterable_attributes)
                .await?;
            info!("Set filterable attributes: {:?}", self.config.filterable_attributes);
        }

        // Configure sortable attributes
        if !self.config.sortable_attributes.is_empty() {
            index
                .set_sortable_attributes(&self.config.sortable_attributes)
                .await?;
            info!("Set sortable attributes: {:?}", self.config.sortable_attributes);
        }

        // Configure ranking rules
        index
            .set_ranking_rules(&self.config.ranking_rules.rules)
            .await?;
        info!("Set ranking rules: {:?}", self.config.ranking_rules.rules);

        // Configure stop words
        if !self.config.stop_words.is_empty() {
            index
                .set_stop_words(&self.config.stop_words)
                .await?;
            info!("Set stop words");
        }

        // Configure synonyms
        if !self.config.synonyms.is_empty() {
            index
                .set_synonyms(&self.config.synonyms)
                .await?;
            info!("Set synonyms");
        }

        info!("Search index initialized successfully");
        Ok(())
    }

    /// Delete and recreate the index (useful for development/testing)
    pub async fn reset_index(&self) -> SearchResult<()> {
        info!("Resetting search index: {}", self.config.index_name);
        
        // Delete index if it exists
        match self.client.delete_index(&self.config.index_name).await {
            Ok(_) => info!("Deleted existing index"),
            Err(_) => info!("No existing index to delete"),
        }
        
        // Wait a moment for deletion to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Recreate index
        self.initialize_index().await?;
        
        Ok(())
    }

    /// Get index statistics
    pub async fn get_stats(&self) -> SearchResult<IndexStats> {
        let index = self.index();
        let stats = index.get_stats().await?;
        
        Ok(IndexStats {
            number_of_documents: stats.number_of_documents,
            is_indexing: stats.is_indexing,
            field_distribution: stats.field_distribution,
        })
    }

    /// Search for torrents
    pub async fn search(&self, query: SearchQuery) -> SearchResult<SearchResults> {
        query.execute(&self.index()).await
    }

    /// Get a single torrent document by ID
    pub async fn get_document(&self, id: uuid::Uuid) -> SearchResult<Option<TorrentDocument>> {
        let index = self.index();
        match index.get_document::<TorrentDocument>(&id.to_string()).await {
            Ok(doc) => Ok(Some(doc)),
            Err(meilisearch_sdk::errors::Error::Meilisearch(e)) if e.error_code == "document_not_found" => {
                Ok(None)
            }
            Err(e) => Err(SearchError::from(e)),
        }
    }

    /// Wait for all pending tasks to complete
    pub async fn wait_for_tasks(&self) -> SearchResult<()> {
        let index = self.index();
        
        // Get all tasks and wait for them
        let tasks = self.client.get_tasks().await?;
        
        for task in tasks.results {
            if !task.is_success() && !task.is_failure() {
                self.client.wait_for_task(task, None, None).await?;
            }
        }
        
        Ok(())
    }

    /// Get the underlying Meilisearch client
    pub fn inner(&self) -> &Client {
        &self.client
    }

    /// Get the search configuration
    pub fn config(&self) -> &SearchConfig {
        &self.config
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub number_of_documents: u64,
    pub is_indexing: bool,
    pub field_distribution: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Meilisearch instance
    async fn test_client_connection() {
        let client = SearchClient::new("http://localhost:7700", "master_key")
            .await
            .unwrap();
        
        let health = client.health_check().await.unwrap();
        assert!(health);
    }

    #[tokio::test]
    #[ignore] // Requires running Meilisearch instance
    async fn test_index_initialization() {
        let client = SearchClient::new("http://localhost:7700", "master_key")
            .await
            .unwrap();
        
        let stats = client.get_stats().await.unwrap();
        assert_eq!(stats.number_of_documents >= 0, true);
    }
}
