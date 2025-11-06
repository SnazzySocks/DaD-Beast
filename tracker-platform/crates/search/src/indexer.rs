//! Indexing operations for Meilisearch

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use crate::schema::TorrentDocument;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Row};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Index operation types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndexOperation {
    /// Add or update a document
    Upsert,
    /// Delete a document
    Delete,
}

/// Queue item from search_index_queue table
#[derive(Debug, Clone)]
struct QueueItem {
    id: i64,
    torrent_id: Uuid,
    operation: IndexOperation,
    created_at: DateTime<Utc>,
}

/// Search indexer for managing torrent documents
pub struct SearchIndexer {
    client: SearchClient,
    db: PgPool,
    batch_size: usize,
    poll_interval: Duration,
}

impl SearchIndexer {
    /// Create a new search indexer
    pub fn new(client: SearchClient, db: PgPool) -> Self {
        Self {
            client,
            db,
            batch_size: 100,
            poll_interval: Duration::from_secs(5),
        }
    }

    /// Create a new search indexer with custom settings
    pub fn with_settings(
        client: SearchClient,
        db: PgPool,
        batch_size: usize,
        poll_interval: Duration,
    ) -> Self {
        Self {
            client,
            db,
            batch_size,
            poll_interval,
        }
    }

    /// Index a single torrent document
    pub async fn index_torrent(&self, torrent_id: Uuid) -> SearchResult<()> {
        let document = self.fetch_torrent_document(torrent_id).await?;
        
        if let Some(doc) = document {
            let index = self.client.index();
            index.add_documents(&[doc], Some("id")).await?;
            info!("Indexed torrent: {}", torrent_id);
        } else {
            warn!("Torrent not found for indexing: {}", torrent_id);
        }
        
        Ok(())
    }

    /// Index multiple torrents in a batch
    pub async fn index_torrents_batch(&self, torrent_ids: Vec<Uuid>) -> SearchResult<usize> {
        if torrent_ids.is_empty() {
            return Ok(0);
        }

        let documents = self.fetch_torrent_documents_batch(torrent_ids).await?;
        let count = documents.len();
        
        if !documents.is_empty() {
            let index = self.client.index();
            index.add_documents(&documents, Some("id")).await?;
            info!("Indexed {} torrents", count);
        }
        
        Ok(count)
    }

    /// Delete a torrent document from the index
    pub async fn delete_torrent(&self, torrent_id: Uuid) -> SearchResult<()> {
        let index = self.client.index();
        index.delete_document(torrent_id.to_string()).await?;
        info!("Deleted torrent from index: {}", torrent_id);
        Ok(())
    }

    /// Delete multiple torrents in a batch
    pub async fn delete_torrents_batch(&self, torrent_ids: Vec<Uuid>) -> SearchResult<usize> {
        if torrent_ids.is_empty() {
            return Ok(0);
        }

        let count = torrent_ids.len();
        let ids: Vec<String> = torrent_ids.into_iter().map(|id| id.to_string()).collect();
        
        let index = self.client.index();
        index.delete_documents(&ids).await?;
        info!("Deleted {} torrents from index", count);
        
        Ok(count)
    }

    /// Process items from the search_index_queue
    pub async fn process_queue(&self) -> SearchResult<usize> {
        let items = self.fetch_queue_items(self.batch_size).await?;
        
        if items.is_empty() {
            return Ok(0);
        }

        let count = items.len();
        debug!("Processing {} queue items", count);

        // Group operations by type
        let mut upsert_ids = Vec::new();
        let mut delete_ids = Vec::new();
        let mut processed_queue_ids = Vec::new();

        for item in items {
            match item.operation {
                IndexOperation::Upsert => upsert_ids.push(item.torrent_id),
                IndexOperation::Delete => delete_ids.push(item.torrent_id),
            }
            processed_queue_ids.push(item.id);
        }

        // Process upserts
        if !upsert_ids.is_empty() {
            if let Err(e) = self.index_torrents_batch(upsert_ids).await {
                error!("Failed to process upsert batch: {}", e);
                return Err(e);
            }
        }

        // Process deletes
        if !delete_ids.is_empty() {
            if let Err(e) = self.delete_torrents_batch(delete_ids).await {
                error!("Failed to process delete batch: {}", e);
                return Err(e);
            }
        }

        // Remove processed items from queue
        self.delete_queue_items(processed_queue_ids).await?;

        Ok(count)
    }

    /// Start background indexing job that continuously processes the queue
    pub async fn start_background_job(self) -> SearchResult<()> {
        info!("Starting background indexing job");
        
        loop {
            match self.process_queue().await {
                Ok(count) => {
                    if count > 0 {
                        debug!("Processed {} items from queue", count);
                    }
                }
                Err(e) => {
                    error!("Error processing queue: {}", e);
                }
            }
            
            sleep(self.poll_interval).await;
        }
    }

    /// Reindex all torrents from the database
    pub async fn reindex_all(&self) -> SearchResult<usize> {
        info!("Starting full reindex of all torrents");
        
        let mut total_indexed = 0;
        let mut offset = 0;

        loop {
            let torrent_ids = self.fetch_all_torrent_ids(self.batch_size, offset).await?;
            
            if torrent_ids.is_empty() {
                break;
            }

            let count = self.index_torrents_batch(torrent_ids.clone()).await?;
            total_indexed += count;
            offset += self.batch_size;

            info!("Reindexed {} torrents (total: {})", count, total_indexed);
            
            // Small delay to avoid overwhelming the database
            sleep(Duration::from_millis(100)).await;
        }

        info!("Full reindex completed: {} torrents", total_indexed);
        Ok(total_indexed)
    }

    /// Clear the entire search index
    pub async fn clear_index(&self) -> SearchResult<()> {
        let index = self.client.index();
        index.delete_all_documents().await?;
        info!("Cleared all documents from search index");
        Ok(())
    }

    // Private helper methods

    /// Fetch a single torrent document from the database
    async fn fetch_torrent_document(&self, torrent_id: Uuid) -> SearchResult<Option<TorrentDocument>> {
        let row = sqlx::query(
            r#"
            SELECT 
                t.id, t.name, t.description, t.info_hash,
                t.category_id, c.name as category,
                t.uploader_id, u.username as uploader,
                t.size, t.seeders, t.leechers, t.snatched,
                t.uploaded_at, t.media_type, t.resolution,
                t.codec, t.quality, t.tmdb_id, t.igdb_id,
                t.year, t.is_freeleech, t.is_double_upload,
                t.is_featured, t.file_count, t.rating,
                t.comment_count,
                ARRAY(SELECT tag FROM torrent_tags WHERE torrent_id = t.id) as tags
            FROM torrents t
            JOIN users u ON t.uploader_id = u.id
            JOIN categories c ON t.category_id = c.id
            WHERE t.id = $1
            "#
        )
        .bind(torrent_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| TorrentDocument {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            info_hash: r.get("info_hash"),
            category: r.get("category"),
            tags: r.get::<Vec<String>, _>("tags"),
            uploader: r.get("uploader"),
            uploader_id: r.get("uploader_id"),
            size: r.get("size"),
            seeders: r.get("seeders"),
            leechers: r.get("leechers"),
            snatched: r.get("snatched"),
            uploaded_at: r.get("uploaded_at"),
            media_type: r.get("media_type"),
            resolution: r.get("resolution"),
            codec: r.get("codec"),
            quality: r.get("quality"),
            tmdb_id: r.get("tmdb_id"),
            igdb_id: r.get("igdb_id"),
            year: r.get("year"),
            is_freeleech: r.get("is_freeleech"),
            is_double_upload: r.get("is_double_upload"),
            is_featured: r.get("is_featured"),
            file_count: r.get("file_count"),
            rating: r.get("rating"),
            comment_count: r.get("comment_count"),
        }))
    }

    /// Fetch multiple torrent documents in a batch
    async fn fetch_torrent_documents_batch(&self, torrent_ids: Vec<Uuid>) -> SearchResult<Vec<TorrentDocument>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                t.id, t.name, t.description, t.info_hash,
                t.category_id, c.name as category,
                t.uploader_id, u.username as uploader,
                t.size, t.seeders, t.leechers, t.snatched,
                t.uploaded_at, t.media_type, t.resolution,
                t.codec, t.quality, t.tmdb_id, t.igdb_id,
                t.year, t.is_freeleech, t.is_double_upload,
                t.is_featured, t.file_count, t.rating,
                t.comment_count,
                ARRAY(SELECT tag FROM torrent_tags WHERE torrent_id = t.id) as tags
            FROM torrents t
            JOIN users u ON t.uploader_id = u.id
            JOIN categories c ON t.category_id = c.id
            WHERE t.id = ANY($1)
            "#
        )
        .bind(&torrent_ids)
        .fetch_all(&self.db)
        .await?;

        let documents = rows
            .into_iter()
            .map(|r| TorrentDocument {
                id: r.get("id"),
                name: r.get("name"),
                description: r.get("description"),
                info_hash: r.get("info_hash"),
                category: r.get("category"),
                tags: r.get::<Vec<String>, _>("tags"),
                uploader: r.get("uploader"),
                uploader_id: r.get("uploader_id"),
                size: r.get("size"),
                seeders: r.get("seeders"),
                leechers: r.get("leechers"),
                snatched: r.get("snatched"),
                uploaded_at: r.get("uploaded_at"),
                media_type: r.get("media_type"),
                resolution: r.get("resolution"),
                codec: r.get("codec"),
                quality: r.get("quality"),
                tmdb_id: r.get("tmdb_id"),
                igdb_id: r.get("igdb_id"),
                year: r.get("year"),
                is_freeleech: r.get("is_freeleech"),
                is_double_upload: r.get("is_double_upload"),
                is_featured: r.get("is_featured"),
                file_count: r.get("file_count"),
                rating: r.get("rating"),
                comment_count: r.get("comment_count"),
            })
            .collect();

        Ok(documents)
    }

    /// Fetch items from the search_index_queue
    async fn fetch_queue_items(&self, limit: usize) -> SearchResult<Vec<QueueItem>> {
        let rows = sqlx::query(
            r#"
            SELECT id, torrent_id, operation, created_at
            FROM search_index_queue
            ORDER BY created_at ASC
            LIMIT $1
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| {
                let operation_str: String = r.get("operation");
                let operation = match operation_str.as_str() {
                    "upsert" => IndexOperation::Upsert,
                    "delete" => IndexOperation::Delete,
                    _ => IndexOperation::Upsert, // Default fallback
                };

                QueueItem {
                    id: r.get("id"),
                    torrent_id: r.get("torrent_id"),
                    operation,
                    created_at: r.get("created_at"),
                }
            })
            .collect();

        Ok(items)
    }

    /// Delete processed items from the queue
    async fn delete_queue_items(&self, ids: Vec<i64>) -> SearchResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        sqlx::query("DELETE FROM search_index_queue WHERE id = ANY($1)")
            .bind(&ids)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Fetch all torrent IDs for reindexing
    async fn fetch_all_torrent_ids(&self, limit: usize, offset: usize) -> SearchResult<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            SELECT id FROM torrents
            ORDER BY id
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.db)
        .await?;

        let ids = rows.into_iter().map(|r| r.get("id")).collect();
        Ok(ids)
    }
}

/// Helper function to queue an index operation
pub async fn queue_index_operation(
    db: &PgPool,
    torrent_id: Uuid,
    operation: IndexOperation,
) -> SearchResult<()> {
    let operation_str = match operation {
        IndexOperation::Upsert => "upsert",
        IndexOperation::Delete => "delete",
    };

    sqlx::query(
        r#"
        INSERT INTO search_index_queue (torrent_id, operation)
        VALUES ($1, $2)
        ON CONFLICT (torrent_id) 
        DO UPDATE SET operation = $2, created_at = NOW()
        "#
    )
    .bind(torrent_id)
    .bind(operation_str)
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_operation_serialization() {
        let op = IndexOperation::Upsert;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"upsert\"");

        let op = IndexOperation::Delete;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"delete\"");
    }
}
