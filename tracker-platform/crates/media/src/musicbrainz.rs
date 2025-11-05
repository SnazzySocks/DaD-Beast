//! MusicBrainz integration for music metadata
//!
//! MusicBrainz is a free, open music encyclopedia that provides metadata
//! for music. The API is free but rate-limited to 1 request per second.

use crate::cache::MetadataCache;
use crate::{ExternalIds, MediaMetadata, MediaType};
use anyhow::{Context, Result};
use governor::{Quota, RateLimiter};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, warn};

const MUSICBRAINZ_API_BASE: &str = "https://musicbrainz.org/ws/2";

/// MusicBrainz client
pub struct MusicBrainzClient {
    http_client: reqwest::Client,
    cache: Arc<MetadataCache>,
    rate_limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
}

impl MusicBrainzClient {
    /// Create a new MusicBrainz client
    pub fn new(user_agent: String, cache: Arc<MetadataCache>) -> Result<Self> {
        // MusicBrainz requires a proper user agent with contact info
        let user_agent = format!("{} (contact: admin@tracker.local)", user_agent);

        let http_client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Rate limit: 1 request per second (MusicBrainz requirement)
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(1).unwrap()),
        ));

        Ok(Self {
            http_client,
            cache,
            rate_limiter,
        })
    }

    /// Search for albums/releases
    pub async fn search_album(&self, query: &str) -> Result<Vec<MediaMetadata>> {
        let cache_key = format!("musicbrainz:search:{}", query);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached MusicBrainz search results");
                return Ok(results);
            }
        }

        self.rate_limiter.until_ready().await;

        let url = format!(
            "{}/release?query={}&fmt=json&limit=10",
            MUSICBRAINZ_API_BASE,
            urlencoding::encode(query)
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to send MusicBrainz API request")?;

        if !response.status().is_success() {
            warn!("MusicBrainz API request failed: {}", response.status());
            return Ok(vec![]);
        }

        let data: MusicBrainzSearchResponse = response.json().await?;

        let results: Vec<MediaMetadata> = data
            .releases
            .into_iter()
            .map(|release| self.convert_release(release))
            .collect();

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Search for artists
    pub async fn search_artist(&self, query: &str) -> Result<Vec<String>> {
        let cache_key = format!("musicbrainz:artist:{}", query);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(results) = serde_json::from_value(cached) {
                debug!("Returning cached MusicBrainz artist search");
                return Ok(results);
            }
        }

        self.rate_limiter.until_ready().await;

        let url = format!(
            "{}/artist?query={}&fmt=json&limit=10",
            MUSICBRAINZ_API_BASE,
            urlencoding::encode(query)
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let data: MusicBrainzArtistResponse = response.json().await?;

        let results: Vec<String> = data.artists.into_iter().map(|a| a.name).collect();

        // Cache results
        if let Ok(value) = serde_json::to_value(&results) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(results)
    }

    /// Get release details by MusicBrainz ID
    pub async fn get_release_details(&self, mbid: &str) -> Result<Option<MediaMetadata>> {
        let cache_key = format!("musicbrainz:release:{}", mbid);

        if let Some(cached) = self.cache.get(&cache_key).await? {
            if let Ok(metadata) = serde_json::from_value(cached) {
                debug!("Returning cached MusicBrainz release details");
                return Ok(Some(metadata));
            }
        }

        self.rate_limiter.until_ready().await;

        let url = format!(
            "{}/release/{}?inc=artists+recordings&fmt=json",
            MUSICBRAINZ_API_BASE, mbid
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("MusicBrainz release details failed: {}", response.status());
            return Ok(None);
        }

        let release: MusicBrainzRelease = response.json().await?;
        let metadata = self.convert_release(release);

        // Cache result
        if let Ok(value) = serde_json::to_value(&metadata) {
            let _ = self.cache.set(&cache_key, &value, None).await;
        }

        Ok(Some(metadata))
    }

    /// Convert MusicBrainz release to MediaMetadata
    fn convert_release(&self, release: MusicBrainzRelease) -> MediaMetadata {
        let artist = release
            .artist_credit
            .as_ref()
            .and_then(|credits| credits.first())
            .map(|c| c.artist.name.clone())
            .unwrap_or_default();

        let title = if artist.is_empty() {
            release.title.clone()
        } else {
            format!("{} - {}", artist, release.title)
        };

        let release_date = release
            .date
            .as_ref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .or_else(|| {
                release
                    .date
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y").ok())
            });

        let artists = release
            .artist_credit
            .as_ref()
            .map(|credits| credits.iter().map(|c| c.artist.name.clone()).collect())
            .unwrap_or_default();

        MediaMetadata {
            media_type: MediaType::Music,
            title,
            original_title: Some(release.title.clone()),
            year: release_date.as_ref().map(|d| d.year()),
            description: None,
            poster_url: None,
            backdrop_url: None,
            rating: None,
            genres: vec![],
            cast: artists,
            crew: vec![],
            runtime: None,
            release_date,
            external_ids: ExternalIds {
                musicbrainz_id: Some(release.id),
                ..Default::default()
            },
        }
    }
}

// API Response types
#[derive(Debug, Deserialize, Serialize)]
struct MusicBrainzSearchResponse {
    releases: Vec<MusicBrainzRelease>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MusicBrainzRelease {
    id: String,
    title: String,
    date: Option<String>,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MusicBrainzArtistCredit>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MusicBrainzArtistCredit {
    artist: MusicBrainzArtist,
}

#[derive(Debug, Deserialize, Serialize)]
struct MusicBrainzArtist {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzArtistResponse {
    artists: Vec<MusicBrainzArtist>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musicbrainz_api_url() {
        assert!(MUSICBRAINZ_API_BASE.starts_with("https://"));
    }
}
