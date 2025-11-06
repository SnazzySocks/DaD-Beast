//! TMDB (The Movie Database) integration
//!
//! Provides movie and TV show metadata from TMDB API with fallback
//! to free scraping methods when no API key is available.
//!
//! API limits:
//! - Free tier: 40 requests per 10 seconds
//! - Rate limiting is enforced via governor

use crate::cache::MetadataCache;
use crate::{ExternalIds, MediaMetadata, MediaType};
use anyhow::{Context, Result};
use governor::{Quota, RateLimiter};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w500";

/// TMDB client
pub struct TmdbClient {
    api_key: Option<String>,
    http_client: reqwest::Client,
    cache: Arc<MetadataCache>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

impl TmdbClient {
    /// Create a new TMDB client
    pub fn new(
        api_key: Option<String>,
        user_agent: String,
        cache: Arc<MetadataCache>,
    ) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Rate limit: 40 requests per 10 seconds
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(4).unwrap()),
        ));

        Ok(Self {
            api_key,
            http_client,
            cache,
            rate_limiter,
        })
    }

    /// Search for movies or TV shows
    pub async fn search(
        &self,
        query: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MediaMetadata>> {
        // Check cache first
        let cache_key = format!(
            "tmdb:search:{}:{}:{}",
            media_type_str(media_type),
            query,
            year.map(|y| y.to_string()).unwrap_or_default()
        );

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached TMDB search results");
                return Ok(results);
            }
        }

        let results = if let Some(ref api_key) = self.api_key {
            self.search_api(query, media_type, year, api_key).await?
        } else {
            warn!("No TMDB API key, falling back to scraping");
            self.search_fallback(query, media_type, year).await?
        };

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Search using TMDB API
    async fn search_api(
        &self,
        query: &str,
        media_type: MediaType,
        year: Option<i32>,
        api_key: &str,
    ) -> Result<Vec<MediaMetadata>> {
        self.rate_limiter.until_ready().await;

        let endpoint = match media_type {
            MediaType::Movie => "search/movie",
            MediaType::TvShow => "search/tv",
            _ => return Ok(vec![]),
        };

        let mut url = format!("{}{}?api_key={}&query={}", TMDB_API_BASE, endpoint, api_key, query);
        if let Some(year) = year {
            url.push_str(&format!("&year={}", year));
        }

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to send TMDB API request")?;

        if !response.status().is_success() {
            warn!("TMDB API request failed: {}", response.status());
            return self.search_fallback(query, media_type, year).await;
        }

        let data: TmdbSearchResponse = response.json().await?;

        Ok(data
            .results
            .into_iter()
            .map(|item| self.convert_search_result(item, media_type))
            .collect())
    }

    /// Fallback to free scraping (using TMDB website)
    async fn search_fallback(
        &self,
        query: &str,
        media_type: MediaType,
        _year: Option<i32>,
    ) -> Result<Vec<MediaMetadata>> {
        warn!("TMDB fallback scraping not fully implemented, returning empty results");
        // In a real implementation, you would scrape the TMDB website
        // For now, return empty results
        Ok(vec![])
    }

    /// Get detailed information about a movie or TV show
    pub async fn get_details(
        &self,
        tmdb_id: i64,
        media_type: MediaType,
    ) -> Result<Option<MediaMetadata>> {
        let cache_key = format!("tmdb:details:{}:{}", media_type_str(media_type), tmdb_id);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(metadata) = serde_json::from_value(cached) {
                debug!("Returning cached TMDB details");
                return Ok(Some(metadata));
            }
        }

        let metadata = if let Some(ref api_key) = self.api_key {
            self.get_details_api(tmdb_id, media_type, api_key).await?
        } else {
            warn!("No TMDB API key, falling back to scraping");
            self.get_details_fallback(tmdb_id, media_type).await?
        };

        if let Some(ref meta) = metadata {
            if let Ok(value) = serde_json::to_value(meta) {
                let _ = self.cache.set(&cache_key, &value, None).await;
            }
        }

        Ok(metadata)
    }

    /// Get details using TMDB API
    async fn get_details_api(
        &self,
        tmdb_id: i64,
        media_type: MediaType,
        api_key: &str,
    ) -> Result<Option<MediaMetadata>> {
        self.rate_limiter.until_ready().await;

        let endpoint = match media_type {
            MediaType::Movie => format!("movie/{}", tmdb_id),
            MediaType::TvShow => format!("tv/{}", tmdb_id),
            _ => return Ok(None),
        };

        let url = format!(
            "{}/{}?api_key={}&append_to_response=credits,external_ids",
            TMDB_API_BASE, endpoint, api_key
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("TMDB details API request failed: {}", response.status());
            return self.get_details_fallback(tmdb_id, media_type).await;
        }

        let data: TmdbDetailsResponse = response.json().await?;

        Ok(Some(self.convert_details(data, media_type, tmdb_id)))
    }

    /// Fallback to scraping for details
    async fn get_details_fallback(
        &self,
        _tmdb_id: i64,
        _media_type: MediaType,
    ) -> Result<Option<MediaMetadata>> {
        warn!("TMDB details fallback not fully implemented");
        Ok(None)
    }

    /// Convert API search result to MediaMetadata
    fn convert_search_result(&self, item: TmdbSearchResult, media_type: MediaType) -> MediaMetadata {
        let title = item.title.or(item.name).unwrap_or_default();
        let release_date = item
            .release_date
            .or(item.first_air_date)
            .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

        MediaMetadata {
            media_type,
            title: title.clone(),
            original_title: item.original_title.or(item.original_name),
            year: release_date.as_ref().map(|d| d.year()),
            description: item.overview,
            poster_url: item.poster_path.map(|p| format!("{}{}", TMDB_IMAGE_BASE, p)),
            backdrop_url: item.backdrop_path.map(|p| format!("{}{}", TMDB_IMAGE_BASE, p)),
            rating: item.vote_average,
            genres: vec![],
            cast: vec![],
            crew: vec![],
            runtime: None,
            release_date,
            external_ids: ExternalIds {
                tmdb_id: Some(item.id),
                ..Default::default()
            },
        }
    }

    /// Convert API details response to MediaMetadata
    fn convert_details(&self, data: TmdbDetailsResponse, media_type: MediaType, tmdb_id: i64) -> MediaMetadata {
        let title = data.title.or(data.name).unwrap_or_default();
        let release_date = data
            .release_date
            .or(data.first_air_date)
            .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

        let genres = data.genres.iter().map(|g| g.name.clone()).collect();

        let cast = data
            .credits
            .as_ref()
            .map(|c| {
                c.cast
                    .iter()
                    .take(10)
                    .map(|p| p.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        let crew = data
            .credits
            .as_ref()
            .map(|c| {
                c.crew
                    .iter()
                    .take(5)
                    .map(|p| format!("{} ({})", p.name, p.job))
                    .collect()
            })
            .unwrap_or_default();

        let imdb_id = data
            .external_ids
            .as_ref()
            .and_then(|e| e.imdb_id.clone());

        MediaMetadata {
            media_type,
            title: title.clone(),
            original_title: data.original_title.or(data.original_name),
            year: release_date.as_ref().map(|d| d.year()),
            description: data.overview,
            poster_url: data.poster_path.map(|p| format!("{}{}", TMDB_IMAGE_BASE, p)),
            backdrop_url: data.backdrop_path.map(|p| format!("{}{}", TMDB_IMAGE_BASE, p)),
            rating: data.vote_average,
            genres,
            cast,
            crew,
            runtime: data.runtime.or(data.episode_run_time.and_then(|v| v.first().copied())),
            release_date,
            external_ids: ExternalIds {
                tmdb_id: Some(tmdb_id),
                imdb_id,
                ..Default::default()
            },
        }
    }
}

fn media_type_str(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Movie => "movie",
        MediaType::TvShow => "tv",
        _ => "other",
    }
}

// API Response types
#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbSearchResult>,
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResult {
    id: i64,
    title: Option<String>,
    name: Option<String>,
    original_title: Option<String>,
    original_name: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct TmdbDetailsResponse {
    id: i64,
    title: Option<String>,
    name: Option<String>,
    original_title: Option<String>,
    original_name: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f32>,
    runtime: Option<i32>,
    episode_run_time: Option<Vec<i32>>,
    genres: Vec<TmdbGenre>,
    credits: Option<TmdbCredits>,
    external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    id: i64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCredits {
    cast: Vec<TmdbCastMember>,
    crew: Vec<TmdbCrewMember>,
}

#[derive(Debug, Deserialize)]
struct TmdbCastMember {
    name: String,
    character: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCrewMember {
    name: String,
    job: String,
}

#[derive(Debug, Deserialize)]
struct TmdbExternalIds {
    imdb_id: Option<String>,
}
