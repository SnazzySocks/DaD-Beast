//! MyAnimeList (MAL) integration for anime and manga metadata
//!
//! MAL has an official API but also supports scraping for free access.
//! This implementation prioritizes scraping to work without API keys.

use crate::cache::MetadataCache;
use crate::{ExternalIds, MediaMetadata, MediaType};
use anyhow::{anyhow, Context, Result};
use governor::{Quota, RateLimiter};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

const MAL_API_BASE: &str = "https://api.myanimelist.net/v2";
const MAL_BASE_URL: &str = "https://myanimelist.net";

/// MyAnimeList client
pub struct MalClient {
    http_client: reqwest::Client,
    cache: Arc<MetadataCache>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

impl MalClient {
    /// Create a new MAL client
    pub fn new(user_agent: String, cache: Arc<MetadataCache>) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Rate limit: 2 requests per second (conservative for scraping)
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(2).unwrap()),
        ));

        Ok(Self {
            http_client,
            cache,
            rate_limiter,
        })
    }

    /// Search for anime or manga
    pub async fn search(&self, query: &str, media_type: MediaType) -> Result<Vec<MediaMetadata>> {
        let type_str = match media_type {
            MediaType::Anime => "anime",
            MediaType::Manga => "manga",
            _ => return Ok(vec![]),
        };

        let cache_key = format!("mal:search:{}:{}", type_str, query);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached MAL search results");
                return Ok(results);
            }
        }

        // Use scraping approach (no API key needed)
        let results = self.search_scrape(query, media_type).await?;

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Search by scraping MAL website
    async fn search_scrape(&self, query: &str, media_type: MediaType) -> Result<Vec<MediaMetadata>> {
        let type_str = match media_type {
            MediaType::Anime => "anime",
            MediaType::Manga => "manga",
            _ => return Ok(vec![]),
        };

        self.rate_limiter.until_ready().await;

        let url = format!(
            "{}/{}.php?q={}",
            MAL_BASE_URL,
            type_str,
            urlencoding::encode(query)
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to scrape MAL search")?;

        if !response.status().is_success() {
            warn!("MAL search scraping failed: {}", response.status());
            return Ok(vec![]);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut results = vec![];

        // Try to parse search results
        // Note: MAL's HTML structure may change, this is a simplified version
        let table_selector = Selector::parse("table").ok();
        let link_selector = Selector::parse("a.hoverinfo_trigger").ok();

        if let (Some(table_sel), Some(link_sel)) = (table_selector, link_selector) {
            for element in document.select(&link_sel) {
                if let Some(href) = element.value().attr("href") {
                    if let Some(title_text) = element.text().next() {
                        // Extract ID from URL like /anime/12345/title
                        let id = extract_mal_id(href);

                        results.push(MediaMetadata {
                            media_type,
                            title: title_text.trim().to_string(),
                            original_title: None,
                            year: None,
                            description: None,
                            poster_url: None,
                            backdrop_url: None,
                            rating: None,
                            genres: vec![],
                            cast: vec![],
                            crew: vec![],
                            runtime: None,
                            release_date: None,
                            external_ids: ExternalIds {
                                mal_id: id,
                                ..Default::default()
                            },
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get details for anime or manga by MAL ID
    pub async fn get_details(&self, mal_id: i64, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        let type_str = match media_type {
            MediaType::Anime => "anime",
            MediaType::Manga => "manga",
            _ => return Ok(None),
        };

        let cache_key = format!("mal:details:{}:{}", type_str, mal_id);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(metadata) = serde_json::from_value(cached) {
                debug!("Returning cached MAL details");
                return Ok(Some(metadata));
            }
        }

        let metadata = self.scrape_details(mal_id, media_type).await?;

        if let Some(ref meta) = metadata {
            if let Ok(value) = serde_json::to_value(meta) {
                let _ = self.cache.set(&cache_key, &value, None).await;
            }
        }

        Ok(metadata)
    }

    /// Scrape details page
    async fn scrape_details(&self, mal_id: i64, media_type: MediaType) -> Result<Option<MediaMetadata>> {
        let type_str = match media_type {
            MediaType::Anime => "anime",
            MediaType::Manga => "manga",
            _ => return Ok(None),
        };

        self.rate_limiter.until_ready().await;

        let url = format!("{}/{}/{}", MAL_BASE_URL, type_str, mal_id);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Parse title
        let title_selector = Selector::parse("h1.title-name").ok();
        let title = title_selector
            .and_then(|sel| document.select(&sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Parse score
        let score_selector = Selector::parse("div.score").ok();
        let rating = score_selector
            .and_then(|sel| document.select(&sel).next())
            .and_then(|e| e.text().collect::<String>().trim().parse::<f32>().ok());

        // Parse synopsis
        let synopsis_selector = Selector::parse("p[itemprop='description']").ok();
        let description = synopsis_selector
            .and_then(|sel| document.select(&sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string());

        // Parse image
        let img_selector = Selector::parse("img[itemprop='image']").ok();
        let poster_url = img_selector
            .and_then(|sel| document.select(&sel).next())
            .and_then(|e| e.value().attr("data-src").or_else(|| e.value().attr("src")))
            .map(|s| s.to_string());

        // Parse genres
        let genre_selector = Selector::parse("span[itemprop='genre']").ok();
        let genres = genre_selector
            .map(|sel| {
                document
                    .select(&sel)
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(MediaMetadata {
            media_type,
            title,
            original_title: None,
            year: None,
            description,
            poster_url,
            backdrop_url: None,
            rating,
            genres,
            cast: vec![],
            crew: vec![],
            runtime: None,
            release_date: None,
            external_ids: ExternalIds {
                mal_id: Some(mal_id),
                ..Default::default()
            },
        }))
    }
}

/// Extract MAL ID from URL
fn extract_mal_id(url: &str) -> Option<i64> {
    // URL format: /anime/12345/title or /manga/12345/title
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 3 {
        parts[2].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mal_id() {
        assert_eq!(extract_mal_id("/anime/12345/title"), Some(12345));
        assert_eq!(extract_mal_id("/manga/67890/title"), Some(67890));
        assert_eq!(extract_mal_id("invalid"), None);
    }

    #[test]
    fn test_mal_urls() {
        assert!(MAL_BASE_URL.starts_with("https://"));
        assert!(MAL_API_BASE.starts_with("https://"));
    }
}
