//! IMDb scraping integration
//!
//! Scrapes IMDb for movie and TV show metadata as a fallback source.
//! No API key required, but must respect robots.txt and rate limits.

use crate::cache::MetadataCache;
use crate::{ExternalIds, MediaMetadata, MediaType};
use anyhow::{anyhow, Context, Result};
use governor::{Quota, RateLimiter};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

const IMDB_BASE_URL: &str = "https://www.imdb.com";
const IMDB_SEARCH_URL: &str = "https://www.imdb.com/find";

/// IMDb client (scraping-based)
pub struct ImdbClient {
    http_client: reqwest::Client,
    cache: Arc<MetadataCache>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

impl ImdbClient {
    /// Create a new IMDb client
    pub fn new(user_agent: String, cache: Arc<MetadataCache>) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Rate limit: 1 request per 2 seconds (conservative for scraping)
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(1).unwrap()).allow_burst(NonZeroU32::new(3).unwrap()),
        ));

        Ok(Self {
            http_client,
            cache,
            rate_limiter,
        })
    }

    /// Search for movies or TV shows
    pub async fn search(&self, query: &str, year: Option<i32>) -> Result<Vec<MediaMetadata>> {
        let cache_key = format!(
            "imdb:search:{}:{}",
            query,
            year.map(|y| y.to_string()).unwrap_or_default()
        );

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached IMDb search results");
                return Ok(results);
            }
        }

        self.rate_limiter.until_ready().await;

        let url = format!("{}/?q={}&s=tt", IMDB_SEARCH_URL, urlencoding::encode(query));

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to scrape IMDb search")?;

        if !response.status().is_success() {
            warn!("IMDb search scraping failed: {}", response.status());
            return Ok(vec![]);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let results = self.parse_search_results(&document, year)?;

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Parse search results from HTML
    fn parse_search_results(&self, document: &Html, target_year: Option<i32>) -> Result<Vec<MediaMetadata>> {
        let mut results = vec![];

        // IMDb search results are in a specific structure
        // Note: IMDb's structure changes frequently, this is a simplified version
        let result_selector = Selector::parse(".find-result-item, .findResult").ok();
        let link_selector = Selector::parse("a").ok();
        let image_selector = Selector::parse("img").ok();

        if let Some(result_sel) = result_selector {
            for element in document.select(&result_sel).take(10) {
                // Extract title and IMDb ID from link
                if let Some(link_sel) = &link_selector {
                    if let Some(link) = element.select(link_sel).next() {
                        if let Some(href) = link.value().attr("href") {
                            if let Some(imdb_id) = extract_imdb_id(href) {
                                let title = link.text().collect::<String>().trim().to_string();

                                if title.is_empty() {
                                    continue;
                                }

                                // Try to extract year from text
                                let text = element.text().collect::<String>();
                                let year = extract_year_from_text(&text);

                                // Filter by year if specified
                                if let (Some(target), Some(found)) = (target_year, year) {
                                    if (target - found).abs() > 1 {
                                        continue;
                                    }
                                }

                                // Try to get poster image
                                let poster_url = if let Some(img_sel) = &image_selector {
                                    element
                                        .select(img_sel)
                                        .next()
                                        .and_then(|img| img.value().attr("src"))
                                        .map(|s| s.to_string())
                                } else {
                                    None
                                };

                                results.push(MediaMetadata {
                                    media_type: MediaType::Other,
                                    title: title.clone(),
                                    original_title: None,
                                    year,
                                    description: None,
                                    poster_url,
                                    backdrop_url: None,
                                    rating: None,
                                    genres: vec![],
                                    cast: vec![],
                                    crew: vec![],
                                    runtime: None,
                                    release_date: year.and_then(|y| {
                                        chrono::NaiveDate::from_ymd_opt(y, 1, 1)
                                    }),
                                    external_ids: ExternalIds {
                                        imdb_id: Some(imdb_id),
                                        ..Default::default()
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get details for a movie or TV show by IMDb ID
    pub async fn get_details(&self, imdb_id: &str) -> Result<Option<MediaMetadata>> {
        let cache_key = format!("imdb:details:{}", imdb_id);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(metadata) = serde_json::from_value(cached) {
                debug!("Returning cached IMDb details");
                return Ok(Some(metadata));
            }
        }

        self.rate_limiter.until_ready().await;

        let url = format!("{}/title/{}/", IMDB_BASE_URL, imdb_id);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("IMDb details scraping failed: {}", response.status());
            return Ok(None);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let metadata = self.parse_details(&document, imdb_id)?;

        if let Some(ref meta) = metadata {
            if let Ok(value) = serde_json::to_value(meta) {
                let _ = self.cache.set(&cache_key, &value, None).await;
            }
        }

        Ok(metadata)
    }

    /// Parse details page HTML
    fn parse_details(&self, document: &Html, imdb_id: &str) -> Result<Option<MediaMetadata>> {
        // Parse title
        let title_selector = Selector::parse("h1, [data-testid='hero-title-block__title']").ok();
        let title = title_selector
            .and_then(|sel| document.select(&sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() {
            return Ok(None);
        }

        // Parse year
        let year_selector = Selector::parse(".sc-8c396aa2-2, [data-testid='hero-title-block__metadata'] a").ok();
        let year = year_selector
            .and_then(|sel| {
                document
                    .select(&sel)
                    .find_map(|e| {
                        let text = e.text().collect::<String>();
                        extract_year_from_text(&text)
                    })
            });

        // Parse rating
        let rating_selector = Selector::parse("[data-testid='hero-rating-bar__aggregate-rating__score'] span").ok();
        let rating = rating_selector
            .and_then(|sel| document.select(&sel).next())
            .and_then(|e| {
                e.text()
                    .collect::<String>()
                    .trim()
                    .parse::<f32>()
                    .ok()
            });

        // Parse plot/description
        let plot_selector = Selector::parse("[data-testid='plot'] span, .summary_text").ok();
        let description = plot_selector
            .and_then(|sel| document.select(&sel).next())
            .map(|e| e.text().collect::<String>().trim().to_string());

        // Parse poster
        let poster_selector = Selector::parse(".ipc-image, .poster img").ok();
        let poster_url = poster_selector
            .and_then(|sel| document.select(&sel).next())
            .and_then(|img| img.value().attr("src"))
            .map(|s| s.to_string());

        // Parse genres
        let genre_selector = Selector::parse("[data-testid='genres'] a, [data-testid='genres'] span").ok();
        let genres = genre_selector
            .map(|sel| {
                document
                    .select(&sel)
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        // Parse cast
        let cast_selector = Selector::parse("[data-testid='title-cast-item'] a").ok();
        let cast = cast_selector
            .map(|sel| {
                document
                    .select(&sel)
                    .take(10)
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(MediaMetadata {
            media_type: MediaType::Other,
            title: title.clone(),
            original_title: None,
            year,
            description,
            poster_url,
            backdrop_url: None,
            rating,
            genres,
            cast,
            crew: vec![],
            runtime: None,
            release_date: year.and_then(|y| chrono::NaiveDate::from_ymd_opt(y, 1, 1)),
            external_ids: ExternalIds {
                imdb_id: Some(imdb_id.to_string()),
                ..Default::default()
            },
        }))
    }
}

/// Extract IMDb ID from URL
fn extract_imdb_id(url: &str) -> Option<String> {
    // URL format: /title/tt1234567/ or /title/tt1234567
    let parts: Vec<&str> = url.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "title" && i + 1 < parts.len() {
            let id = parts[i + 1];
            if id.starts_with("tt") {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Extract year from text
fn extract_year_from_text(text: &str) -> Option<i32> {
    let re = regex::Regex::new(r"\b(19\d{2}|20\d{2})\b").ok()?;
    re.captures(text)?
        .get(1)?
        .as_str()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_imdb_id() {
        assert_eq!(
            extract_imdb_id("/title/tt0111161/"),
            Some("tt0111161".to_string())
        );
        assert_eq!(
            extract_imdb_id("/title/tt1234567"),
            Some("tt1234567".to_string())
        );
        assert_eq!(extract_imdb_id("/invalid/path"), None);
    }

    #[test]
    fn test_extract_year_from_text() {
        assert_eq!(extract_year_from_text("Movie Title (2023)"), Some(2023));
        assert_eq!(extract_year_from_text("1999-2004"), Some(1999));
        assert_eq!(extract_year_from_text("No year here"), None);
    }

    #[test]
    fn test_imdb_urls() {
        assert!(IMDB_BASE_URL.starts_with("https://"));
        assert!(IMDB_SEARCH_URL.starts_with("https://"));
    }
}
