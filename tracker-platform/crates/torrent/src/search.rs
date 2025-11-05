//! Search integration with Meilisearch
//!
//! This module handles:
//! - Queue torrents for Meilisearch indexing
//! - Extract searchable fields
//! - Category/tag extraction
//! - Update search index on changes

use anyhow::{Context, Result};
use meilisearch_sdk::{Client, Index};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Searchable torrent document for Meilisearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSearchDocument {
    /// Torrent ID (primary key)
    pub id: String,

    /// Torrent name
    pub name: String,

    /// Description (full text)
    pub description: Option<String>,

    /// Category name
    pub category: String,

    /// Category ID
    pub category_id: String,

    /// Tags
    pub tags: Vec<String>,

    /// Info hash
    pub info_hash: String,

    /// Total size in bytes
    pub total_size: i64,

    /// Total size formatted (e.g., "1.5 GB")
    pub total_size_formatted: String,

    /// File count
    pub file_count: i32,

    /// Uploader username
    pub uploader: String,

    /// Uploader ID
    pub uploader_id: String,

    /// Media type
    pub media_type: String,

    /// Quality information (resolution, codec, source)
    pub quality: Option<QualitySearchInfo>,

    /// External IDs for linking
    pub external_ids: ExternalIdsSearch,

    /// Release year
    pub year: Option<i32>,

    /// IMDB rating
    pub imdb_rating: Option<f32>,

    /// Seeders count
    pub seeders: i32,

    /// Leechers count
    pub leechers: i32,

    /// Times completed
    pub times_completed: i32,

    /// Upload timestamp (Unix)
    pub created_at: i64,

    /// Is freeleech
    pub is_freeleech: bool,

    /// Is featured
    pub is_featured: bool,

    /// Is sticky
    pub is_sticky: bool,
}

/// Quality information for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySearchInfo {
    pub resolution: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub source: Option<String>,
}

/// External IDs for search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalIdsSearch {
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
    pub igdb_id: Option<i64>,
}

/// Search service
pub struct SearchService {
    pool: PgPool,
    client: Client,
    index_name: String,
}

impl SearchService {
    /// Create new search service
    pub fn new(pool: PgPool, meilisearch_url: &str, api_key: &str, index_name: String) -> Self {
        let client = Client::new(meilisearch_url, Some(api_key));
        Self {
            pool,
            client,
            index_name,
        }
    }

    /// Initialize Meilisearch index with proper settings
    pub async fn initialize_index(&self) -> Result<()> {
        let index = self.get_index().await?;

        // Set searchable attributes
        index
            .set_searchable_attributes(&[
                "name",
                "description",
                "tags",
                "category",
                "uploader",
                "info_hash",
            ])
            .await
            .context("Failed to set searchable attributes")?;

        // Set filterable attributes
        index
            .set_filterable_attributes(&[
                "category_id",
                "media_type",
                "uploader_id",
                "year",
                "is_freeleech",
                "is_featured",
                "tags",
                "quality.resolution",
                "quality.source",
                "seeders",
                "leechers",
            ])
            .await
            .context("Failed to set filterable attributes")?;

        // Set sortable attributes
        index
            .set_sortable_attributes(&[
                "created_at",
                "total_size",
                "seeders",
                "leechers",
                "times_completed",
                "imdb_rating",
            ])
            .await
            .context("Failed to set sortable attributes")?;

        // Set ranking rules
        index
            .set_ranking_rules(&[
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
                "seeders:desc",
            ])
            .await
            .context("Failed to set ranking rules")?;

        Ok(())
    }

    /// Get Meilisearch index
    async fn get_index(&self) -> Result<Index> {
        Ok(self.client.index(&self.index_name))
    }

    /// Index a single torrent
    pub async fn index_torrent(&self, torrent_id: Uuid) -> Result<()> {
        let document = self.fetch_torrent_document(torrent_id).await?;
        let index = self.get_index().await?;

        index
            .add_documents(&[document], Some("id"))
            .await
            .context("Failed to add document to index")?;

        Ok(())
    }

    /// Index multiple torrents
    pub async fn index_torrents(&self, torrent_ids: Vec<Uuid>) -> Result<()> {
        let mut documents = Vec::new();

        for torrent_id in torrent_ids {
            match self.fetch_torrent_document(torrent_id).await {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    tracing::error!("Failed to fetch torrent {}: {}", torrent_id, e);
                }
            }
        }

        if !documents.is_empty() {
            let index = self.get_index().await?;
            index
                .add_documents(&documents, Some("id"))
                .await
                .context("Failed to add documents to index")?;
        }

        Ok(())
    }

    /// Remove torrent from index
    pub async fn remove_torrent(&self, torrent_id: Uuid) -> Result<()> {
        let index = self.get_index().await?;

        index
            .delete_document(&torrent_id.to_string())
            .await
            .context("Failed to remove document from index")?;

        Ok(())
    }

    /// Update torrent in index
    pub async fn update_torrent(&self, torrent_id: Uuid) -> Result<()> {
        // Same as indexing - Meilisearch handles updates
        self.index_torrent(torrent_id).await
    }

    /// Fetch torrent data and convert to search document
    async fn fetch_torrent_document(&self, torrent_id: Uuid) -> Result<TorrentSearchDocument> {
        let record = sqlx::query!(
            r#"
            SELECT
                t.id, t.name, t.info_hash, t.total_size, t.created_at,
                tm.description, tm.tags, tm.year, tm.imdb_rating,
                tm.quality, tm.external_ids, tm.is_featured, tm.is_sticky,
                tm.media_type as "media_type: crate::metadata::MediaType",
                c.name as category_name, c.id as category_id,
                u.username as uploader_name, u.id as uploader_id,
                t.freeleech_type as "freeleech_type: crate::download::FreeleechType",
                COUNT(DISTINCT tf.id)::int as file_count,
                COALESCE(ts.seeders, 0)::int as seeders,
                COALESCE(ts.leechers, 0)::int as leechers,
                t.times_completed
            FROM torrents t
            JOIN torrent_metadata tm ON tm.id = t.id
            JOIN categories c ON c.id = t.category_id
            JOIN users u ON u.id = t.uploader_id
            LEFT JOIN torrent_files tf ON tf.torrent_id = t.id
            LEFT JOIN torrent_stats ts ON ts.torrent_id = t.id
            WHERE t.id = $1
            AND t.moderation_status = 'approved'
            GROUP BY t.id, tm.id, c.id, u.id, ts.seeders, ts.leechers
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch torrent data")?;

        // Parse quality
        let quality: Option<crate::metadata::QualityInfo> = record
            .quality
            .and_then(|v| serde_json::from_value(v).ok());

        let quality_search = quality.map(|q| QualitySearchInfo {
            resolution: q.resolution,
            video_codec: q.video_codec,
            audio_codec: q.audio_codec,
            source: q.source,
        });

        // Parse external IDs
        let external_ids: crate::metadata::ExternalIds = record
            .external_ids
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let external_ids_search = ExternalIdsSearch {
            tmdb_id: external_ids.tmdb_id,
            imdb_id: external_ids.imdb_id,
            tvdb_id: external_ids.tvdb_id,
            igdb_id: external_ids.igdb_id,
        };

        // Format size
        let total_size_formatted = format_size(record.total_size);

        // Check freeleech
        let is_freeleech = record
            .freeleech_type
            .map(|ft| ft != crate::download::FreeleechType::None)
            .unwrap_or(false);

        Ok(TorrentSearchDocument {
            id: record.id.to_string(),
            name: record.name,
            description: record.description,
            category: record.category_name,
            category_id: record.category_id.to_string(),
            tags: record.tags,
            info_hash: record.info_hash,
            total_size: record.total_size,
            total_size_formatted,
            file_count: record.file_count.unwrap_or(0),
            uploader: record.uploader_name,
            uploader_id: record.uploader_id.to_string(),
            media_type: format!("{:?}", record.media_type),
            quality: quality_search,
            external_ids: external_ids_search,
            year: record.year,
            imdb_rating: record.imdb_rating,
            seeders: record.seeders.unwrap_or(0),
            leechers: record.leechers.unwrap_or(0),
            times_completed: record.times_completed,
            created_at: record.created_at.timestamp(),
            is_freeleech,
            is_featured: record.is_featured,
            is_sticky: record.is_sticky,
        })
    }

    /// Process indexing queue
    pub async fn process_queue(&self, batch_size: i64) -> Result<usize> {
        // Get pending items from queue
        let queue_items = sqlx::query!(
            r#"
            SELECT id, torrent_id
            FROM search_index_queue
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            batch_size
        )
        .fetch_all(&self.pool)
        .await?;

        let count = queue_items.len();

        if count == 0 {
            return Ok(0);
        }

        // Extract torrent IDs
        let torrent_ids: Vec<Uuid> = queue_items.iter().map(|item| item.torrent_id).collect();

        // Index torrents
        self.index_torrents(torrent_ids.clone()).await?;

        // Mark as processed
        for item in queue_items {
            sqlx::query!(
                r#"
                UPDATE search_index_queue
                SET status = 'completed', processed_at = NOW()
                WHERE id = $1
                "#,
                item.id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(count)
    }

    /// Reindex all approved torrents
    pub async fn reindex_all(&self) -> Result<usize> {
        let torrent_ids = sqlx::query!(
            r#"
            SELECT id
            FROM torrents
            WHERE moderation_status = 'approved'
            AND is_deleted = false
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let count = torrent_ids.len();

        let ids: Vec<Uuid> = torrent_ids.into_iter().map(|r| r.id).collect();

        // Process in batches of 100
        for chunk in ids.chunks(100) {
            self.index_torrents(chunk.to_vec()).await?;
        }

        Ok(count)
    }
}

/// Format file size for display
fn format_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f64 = bytes as f64;
    let exponent = (bytes_f64.log10() / 3.0).floor() as usize;
    let exponent = exponent.min(UNITS.len() - 1);

    let value = bytes_f64 / 1000_f64.powi(exponent as i32);

    format!("{:.2} {}", value, UNITS[exponent])
}

/// Queue torrent for indexing
pub async fn queue_for_indexing(torrent_id: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO search_index_queue (id, torrent_id, status)
        VALUES ($1, $2, 'pending')
        ON CONFLICT (torrent_id) DO UPDATE
        SET status = 'pending', updated_at = NOW()
        "#,
        Uuid::new_v4(),
        torrent_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Queue torrent for removal from index
pub async fn queue_for_removal(torrent_id: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO search_index_queue (id, torrent_id, status)
        VALUES ($1, $2, 'removal')
        ON CONFLICT (torrent_id) DO UPDATE
        SET status = 'removal', updated_at = NOW()
        "#,
        Uuid::new_v4(),
        torrent_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500.00 B");
        assert_eq!(format_size(1500), "1.50 KB");
        assert_eq!(format_size(1500000), "1.50 MB");
        assert_eq!(format_size(1500000000), "1.50 GB");
        assert_eq!(format_size(1500000000000), "1.50 TB");
    }
}
