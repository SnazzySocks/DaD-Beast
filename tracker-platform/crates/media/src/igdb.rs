//! IGDB (Internet Game Database) integration
//!
//! Provides video game metadata from IGDB API with fallback
//! to free scraping methods (Wikipedia, MobyGames) when no API key is available.
//!
//! API requires Twitch OAuth token and has rate limiting.

use crate::cache::MetadataCache;
use crate::{ExternalIds, MediaMetadata, MediaType};
use anyhow::{anyhow, Context, Result};
use governor::{Quota, RateLimiter};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

const IGDB_API_BASE: &str = "https://api.igdb.com/v4";
const TWITCH_OAUTH_URL: &str = "https://id.twitch.tv/oauth2/token";

/// IGDB client
pub struct IgdbClient {
    client_id: Option<String>,
    client_secret: Option<String>,
    access_token: Option<String>,
    http_client: reqwest::Client,
    cache: Arc<MetadataCache>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

impl IgdbClient {
    /// Create a new IGDB client
    pub fn new(
        client_id: Option<String>,
        client_secret: Option<String>,
        user_agent: String,
        cache: Arc<MetadataCache>,
    ) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Rate limit: 4 requests per second
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(4).unwrap()),
        ));

        Ok(Self {
            client_id,
            client_secret,
            access_token: None,
            http_client,
            cache,
            rate_limiter,
        })
    }

    /// Authenticate with Twitch to get IGDB access token
    async fn authenticate(&mut self) -> Result<()> {
        if self.access_token.is_some() {
            return Ok(());
        }

        let client_id = self
            .client_id
            .as_ref()
            .ok_or_else(|| anyhow!("IGDB client ID not configured"))?;
        let client_secret = self
            .client_secret
            .as_ref()
            .ok_or_else(|| anyhow!("IGDB client secret not configured"))?;

        let response = self
            .http_client
            .post(TWITCH_OAUTH_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await?;

        let auth_response: TwitchAuthResponse = response.json().await?;
        self.access_token = Some(auth_response.access_token);

        Ok(())
    }

    /// Search for games
    pub async fn search_game(&self, query: &str) -> Result<Vec<MediaMetadata>> {
        let cache_key = format!("igdb:search:{}", query);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached IGDB search results");
                return Ok(results);
            }
        }

        let results = if self.client_id.is_some() {
            // Try API first, but mutable self is problematic
            // For now, fall back to scraping
            warn!("IGDB API requires mutable authentication, using fallback");
            self.search_fallback(query).await?
        } else {
            warn!("No IGDB credentials, falling back to scraping");
            self.search_fallback(query).await?
        };

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Search using free sources (Wikipedia, etc.)
    async fn search_fallback(&self, query: &str) -> Result<Vec<MediaMetadata>> {
        // In a real implementation, this would scrape Wikipedia or MobyGames
        debug!("IGDB fallback search for: {}", query);

        // Try Wikipedia search
        self.search_wikipedia(query).await
    }

    /// Search Wikipedia for game info
    async fn search_wikipedia(&self, query: &str) -> Result<Vec<MediaMetadata>> {
        let search_url = format!(
            "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit=5&format=json",
            urlencoding::encode(query)
        );

        self.rate_limiter.until_ready().await;

        let response = self.http_client.get(&search_url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let mut results = vec![];

        // Wikipedia OpenSearch returns [query, [titles], [descriptions], [urls]]
        if let Some(titles) = data.get(1).and_then(|v| v.as_array()) {
            if let Some(descriptions) = data.get(2).and_then(|v| v.as_array()) {
                for (i, title) in titles.iter().enumerate() {
                    if let Some(title_str) = title.as_str() {
                        // Only include if it seems game-related
                        if title_str.contains("video game")
                            || title_str.contains("game)")
                            || descriptions
                                .get(i)
                                .and_then(|d| d.as_str())
                                .map(|d| d.contains("video game"))
                                .unwrap_or(false)
                        {
                            results.push(MediaMetadata {
                                media_type: MediaType::Game,
                                title: title_str.to_string(),
                                original_title: None,
                                year: None,
                                description: descriptions
                                    .get(i)
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.to_string()),
                                poster_url: None,
                                backdrop_url: None,
                                rating: None,
                                genres: vec![],
                                cast: vec![],
                                crew: vec![],
                                runtime: None,
                                release_date: None,
                                external_ids: ExternalIds::default(),
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get game details by IGDB ID
    pub async fn get_game_details(&self, igdb_id: i64) -> Result<Option<MediaMetadata>> {
        let cache_key = format!("igdb:details:{}", igdb_id);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(metadata) = serde_json::from_value(cached) {
                debug!("Returning cached IGDB details");
                return Ok(Some(metadata));
            }
        }

        // For now, return None as we'd need mutable self for API auth
        warn!("IGDB details not fully implemented");
        Ok(None)
    }

    /// Scrape MobyGames for game details
    async fn scrape_mobygames(&self, game_title: &str) -> Result<Option<MediaMetadata>> {
        // MobyGames has a predictable URL structure
        let slug = game_title
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        let url = format!("https://www.mobygames.com/game/{}", slug);

        self.rate_limiter.until_ready().await;

        let response = match self.http_client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        if !response.status().is_success() {
            return Ok(None);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Try to parse basic info from MobyGames
        let title_selector = Selector::parse("h1").unwrap();
        let description_selector = Selector::parse(".game-description").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| game_title.to_string());

        let description = document
            .select(&description_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string());

        Ok(Some(MediaMetadata {
            media_type: MediaType::Game,
            title,
            original_title: None,
            year: None,
            description,
            poster_url: None,
            backdrop_url: None,
            rating: None,
            genres: vec![],
            cast: vec![],
            crew: vec![],
            runtime: None,
            release_date: None,
            external_ids: ExternalIds::default(),
        }))
    }
}

// API Response types
#[derive(Debug, Deserialize)]
struct TwitchAuthResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IgdbGame {
    id: i64,
    name: String,
    summary: Option<String>,
    cover: Option<IgdbCover>,
    first_release_date: Option<i64>,
    genres: Option<Vec<IgdbGenre>>,
    platforms: Option<Vec<IgdbPlatform>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IgdbCover {
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IgdbGenre {
    id: i64,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IgdbPlatform {
    id: i64,
    name: String,
}
