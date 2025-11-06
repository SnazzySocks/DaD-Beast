//! Automatic torrent enrichment with media metadata
//!
//! This module provides background jobs to automatically enrich torrents
//! with metadata from various sources based on their names.
//!
//! Workflow:
//! 1. Detect media type from torrent name
//! 2. Extract title, year, and other info
//! 3. Search appropriate metadata source
//! 4. Store metadata in database
//! 5. Queue torrent for search reindexing

use crate::detector::{detect_media_info, MediaInfo};
use crate::{MediaMetadata, MediaService, MediaType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Enrichment status for a torrent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "enrichment_status", rename_all = "lowercase")]
pub enum EnrichmentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Enrichment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub torrent_id: Uuid,
    pub status: EnrichmentStatus,
    pub metadata: Option<MediaMetadata>,
    pub media_info: MediaInfo,
    pub error: Option<String>,
}

/// Enrich a single torrent with metadata
pub async fn enrich_torrent(
    service: &MediaService,
    torrent_id: Uuid,
    torrent_name: &str,
) -> Result<Option<MediaMetadata>> {
    info!("Enriching torrent: {} ({})", torrent_id, torrent_name);

    // Step 1: Detect media type and extract info
    let media_info = detect_media_info(torrent_name);
    debug!("Detected media info: {:?}", media_info);

    if media_info.title.is_empty() {
        warn!("Could not extract title from torrent name");
        return Ok(None);
    }

    // Step 2: Search for metadata
    let search_results = service
        .search(&media_info.title, media_info.media_type, media_info.year)
        .await
        .context("Failed to search for metadata")?;

    if search_results.is_empty() {
        info!("No metadata found for: {}", media_info.title);
        return Ok(None);
    }

    // Step 3: Pick the best match
    let best_match = pick_best_match(&search_results, &media_info);

    // Step 4: Store in database
    if let Some(metadata) = &best_match {
        store_enrichment(service.db(), torrent_id, metadata)
            .await
            .context("Failed to store enrichment")?;

        info!(
            "Successfully enriched torrent {} with metadata for '{}'",
            torrent_id, metadata.title
        );
    }

    Ok(best_match)
}

/// Pick the best matching metadata from search results
fn pick_best_match(results: &[MediaMetadata], media_info: &MediaInfo) -> Option<MediaMetadata> {
    if results.is_empty() {
        return None;
    }

    // If we have a year, try to match it
    if let Some(target_year) = media_info.year {
        for result in results {
            if let Some(result_year) = result.year {
                // Allow 1 year difference for potential release date variations
                if (target_year - result_year).abs() <= 1 {
                    return Some(result.clone());
                }
            }
        }
    }

    // If no year match, try title similarity
    let normalized_search = normalize_title(&media_info.title);

    for result in results {
        let normalized_result = normalize_title(&result.title);
        if normalized_search == normalized_result {
            return Some(result.clone());
        }
    }

    // Fall back to first result
    Some(results[0].clone())
}

/// Normalize title for comparison
fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Store enrichment data in database
async fn store_enrichment(
    db: &PgPool,
    torrent_id: Uuid,
    metadata: &MediaMetadata,
) -> Result<()> {
    let metadata_json = serde_json::to_value(metadata)?;

    sqlx::query!(
        r#"
        INSERT INTO torrent_metadata (torrent_id, media_type, metadata, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (torrent_id) DO UPDATE
        SET media_type = $2, metadata = $3, updated_at = NOW()
        "#,
        torrent_id,
        serde_json::to_string(&metadata.media_type)?,
        metadata_json
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Batch enrich multiple torrents
pub async fn batch_enrich(
    service: &MediaService,
    torrent_ids: &[Uuid],
) -> Result<Vec<EnrichmentResult>> {
    let mut results = vec![];

    for torrent_id in torrent_ids {
        // Fetch torrent name from database
        let torrent = match get_torrent_name(service.db(), *torrent_id).await {
            Ok(Some(name)) => name,
            Ok(None) => {
                warn!("Torrent not found: {}", torrent_id);
                continue;
            }
            Err(e) => {
                error!("Failed to fetch torrent {}: {}", torrent_id, e);
                continue;
            }
        };

        let media_info = detect_media_info(&torrent);

        match enrich_torrent(service, *torrent_id, &torrent).await {
            Ok(Some(metadata)) => {
                results.push(EnrichmentResult {
                    torrent_id: *torrent_id,
                    status: EnrichmentStatus::Completed,
                    metadata: Some(metadata),
                    media_info,
                    error: None,
                });
            }
            Ok(None) => {
                results.push(EnrichmentResult {
                    torrent_id: *torrent_id,
                    status: EnrichmentStatus::Failed,
                    metadata: None,
                    media_info,
                    error: Some("No metadata found".to_string()),
                });
            }
            Err(e) => {
                error!("Failed to enrich torrent {}: {}", torrent_id, e);
                results.push(EnrichmentResult {
                    torrent_id: *torrent_id,
                    status: EnrichmentStatus::Failed,
                    metadata: None,
                    media_info,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

/// Get torrent name by ID
async fn get_torrent_name(db: &PgPool, torrent_id: Uuid) -> Result<Option<String>> {
    let result = sqlx::query!(
        r#"
        SELECT name FROM torrents WHERE id = $1
        "#,
        torrent_id
    )
    .fetch_optional(db)
    .await?;

    Ok(result.map(|r| r.name))
}

/// Get pending enrichment jobs
pub async fn get_pending_enrichments(db: &PgPool, limit: i64) -> Result<Vec<Uuid>> {
    let results = sqlx::query!(
        r#"
        SELECT t.id
        FROM torrents t
        LEFT JOIN torrent_metadata tm ON t.id = tm.torrent_id
        WHERE tm.torrent_id IS NULL
        ORDER BY t.created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db)
    .await?;

    Ok(results.into_iter().map(|r| r.id).collect())
}

/// Retry failed enrichments
pub async fn retry_failed_enrichments(
    service: &MediaService,
    max_retries: i32,
    limit: i64,
) -> Result<Vec<EnrichmentResult>> {
    // This would query for failed enrichments and retry them
    // For now, just return empty
    Ok(vec![])
}

/// Background job to continuously enrich torrents
pub async fn enrichment_worker(service: MediaService, batch_size: i64) -> Result<()> {
    info!("Starting enrichment worker (batch size: {})", batch_size);

    loop {
        // Get pending torrents
        match get_pending_enrichments(service.db(), batch_size).await {
            Ok(torrent_ids) => {
                if torrent_ids.is_empty() {
                    debug!("No pending enrichments, sleeping...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    continue;
                }

                info!("Processing {} pending enrichments", torrent_ids.len());

                // Batch enrich
                match batch_enrich(&service, &torrent_ids).await {
                    Ok(results) => {
                        let completed = results
                            .iter()
                            .filter(|r| r.status == EnrichmentStatus::Completed)
                            .count();
                        info!(
                            "Enrichment batch completed: {}/{} successful",
                            completed,
                            results.len()
                        );
                    }
                    Err(e) => {
                        error!("Batch enrichment failed: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch pending enrichments: {}", e);
            }
        }

        // Sleep between batches
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(
            normalize_title("The Matrix (1999)"),
            "the matrix 1999"
        );
        assert_eq!(
            normalize_title("Breaking.Bad.S01E01"),
            "breaking bad s01e01"
        );
    }

    #[test]
    fn test_pick_best_match() {
        let media_info = MediaInfo {
            media_type: MediaType::Movie,
            title: "The Matrix".to_string(),
            year: Some(1999),
            ..Default::default()
        };

        let results = vec![
            MediaMetadata {
                media_type: MediaType::Movie,
                title: "The Matrix".to_string(),
                year: Some(1999),
                original_title: None,
                description: None,
                poster_url: None,
                backdrop_url: None,
                rating: None,
                genres: vec![],
                cast: vec![],
                crew: vec![],
                runtime: None,
                release_date: None,
                external_ids: Default::default(),
            },
            MediaMetadata {
                media_type: MediaType::Movie,
                title: "The Matrix Reloaded".to_string(),
                year: Some(2003),
                original_title: None,
                description: None,
                poster_url: None,
                backdrop_url: None,
                rating: None,
                genres: vec![],
                cast: vec![],
                crew: vec![],
                runtime: None,
                release_date: None,
                external_ids: Default::default(),
            },
        ];

        let best = pick_best_match(&results, &media_info);
        assert!(best.is_some());
        assert_eq!(best.unwrap().year, Some(1999));
    }
}
