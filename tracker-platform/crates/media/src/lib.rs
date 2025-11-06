//! Media metadata scraping service
//!
//! This crate provides comprehensive media metadata scraping from multiple sources:
//! - TMDB (The Movie Database) for movies and TV shows
//! - IGDB (Internet Game Database) for video games
//! - MusicBrainz for music
//! - MyAnimeList for anime and manga
//! - IMDb scraping as fallback
//!
//! Features:
//! - API key fallback to free scraping methods
//! - Aggressive caching to minimize external requests
//! - Rate limiting to respect API quotas
//! - Automatic media type detection from torrent names
//! - Background enrichment jobs

pub mod cache;
pub mod detector;
pub mod enricher;
pub mod igdb;
pub mod imdb;
pub mod mal;
pub mod musicbrainz;
pub mod tmdb;

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;

/// Media types supported by the platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    TvShow,
    Game,
    Music,
    Anime,
    Manga,
    Other,
}

impl MediaType {
    /// Parse media type from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "movie" | "movies" | "film" => MediaType::Movie,
            "tv" | "tvshow" | "tv-show" | "series" => MediaType::TvShow,
            "game" | "games" | "videogame" => MediaType::Game,
            "music" | "audio" | "album" => MediaType::Music,
            "anime" => MediaType::Anime,
            "manga" => MediaType::Manga,
            _ => MediaType::Other,
        }
    }
}

/// Metadata for any media type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaMetadata {
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub rating: Option<f32>,
    pub genres: Vec<String>,
    pub cast: Vec<String>,
    pub crew: Vec<String>,
    pub runtime: Option<i32>,
    pub release_date: Option<chrono::NaiveDate>,
    pub external_ids: ExternalIds,
}

/// External IDs for cross-referencing
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExternalIds {
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub igdb_id: Option<i64>,
    pub mal_id: Option<i64>,
    pub musicbrainz_id: Option<String>,
}

/// Configuration for media scrapers
#[derive(Debug, Clone)]
pub struct MediaConfig {
    pub tmdb_api_key: Option<String>,
    pub igdb_client_id: Option<String>,
    pub igdb_client_secret: Option<String>,
    pub enable_scraping_fallback: bool,
    pub user_agent: String,
    pub cache_ttl_days: i32,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            tmdb_api_key: None,
            igdb_client_id: None,
            igdb_client_secret: None,
            enable_scraping_fallback: true,
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
            cache_ttl_days: 30,
        }
    }
}

/// Main media service that coordinates all scrapers
pub struct MediaService {
    config: MediaConfig,
    db: PgPool,
    tmdb_client: tmdb::TmdbClient,
    igdb_client: igdb::IgdbClient,
    musicbrainz_client: musicbrainz::MusicBrainzClient,
    mal_client: mal::MalClient,
    imdb_client: imdb::ImdbClient,
    cache: Arc<cache::MetadataCache>,
}

impl MediaService {
    /// Create a new media service
    pub fn new(config: MediaConfig, db: PgPool) -> Result<Self> {
        let cache = Arc::new(cache::MetadataCache::new(db.clone()));

        Ok(Self {
            tmdb_client: tmdb::TmdbClient::new(
                config.tmdb_api_key.clone(),
                config.user_agent.clone(),
                cache.clone(),
            )?,
            igdb_client: igdb::IgdbClient::new(
                config.igdb_client_id.clone(),
                config.igdb_client_secret.clone(),
                config.user_agent.clone(),
                cache.clone(),
            )?,
            musicbrainz_client: musicbrainz::MusicBrainzClient::new(
                config.user_agent.clone(),
                cache.clone(),
            )?,
            mal_client: mal::MalClient::new(
                config.user_agent.clone(),
                cache.clone(),
            )?,
            imdb_client: imdb::ImdbClient::new(
                config.user_agent.clone(),
                cache.clone(),
            )?,
            cache,
            config,
            db,
        })
    }

    /// Search for media by title and optional year
    pub async fn search(
        &self,
        title: &str,
        media_type: MediaType,
        year: Option<i32>,
    ) -> Result<Vec<MediaMetadata>> {
        match media_type {
            MediaType::Movie | MediaType::TvShow => {
                self.tmdb_client.search(title, media_type, year).await
            }
            MediaType::Game => self.igdb_client.search_game(title).await,
            MediaType::Music => self.musicbrainz_client.search_album(title).await,
            MediaType::Anime | MediaType::Manga => {
                self.mal_client.search(title, media_type).await
            }
            MediaType::Other => {
                // Try IMDb scraping as fallback
                self.imdb_client.search(title, year).await
            }
        }
    }

    /// Get detailed metadata by external ID
    pub async fn get_by_id(
        &self,
        media_type: MediaType,
        external_id: &str,
    ) -> Result<Option<MediaMetadata>> {
        match media_type {
            MediaType::Movie | MediaType::TvShow => {
                if let Ok(tmdb_id) = external_id.parse::<i64>() {
                    self.tmdb_client.get_details(tmdb_id, media_type).await
                } else {
                    Ok(None)
                }
            }
            MediaType::Game => {
                if let Ok(igdb_id) = external_id.parse::<i64>() {
                    self.igdb_client.get_game_details(igdb_id).await
                } else {
                    Ok(None)
                }
            }
            MediaType::Music => {
                self.musicbrainz_client.get_release_details(external_id).await
            }
            MediaType::Anime | MediaType::Manga => {
                if let Ok(mal_id) = external_id.parse::<i64>() {
                    self.mal_client.get_details(mal_id, media_type).await
                } else {
                    Ok(None)
                }
            }
            MediaType::Other => {
                // Try IMDb
                if external_id.starts_with("tt") {
                    self.imdb_client.get_details(external_id).await
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Detect media type and extract info from torrent name
    pub fn detect_media_info(&self, torrent_name: &str) -> detector::MediaInfo {
        detector::detect_media_info(torrent_name)
    }

    /// Enrich a torrent with metadata
    pub async fn enrich_torrent(
        &self,
        torrent_id: uuid::Uuid,
        torrent_name: &str,
    ) -> Result<Option<MediaMetadata>> {
        enricher::enrich_torrent(self, torrent_id, torrent_name).await
    }

    /// Get database pool
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    /// Get cache
    pub fn cache(&self) -> Arc<cache::MetadataCache> {
        self.cache.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_str() {
        assert_eq!(MediaType::from_str("movie"), MediaType::Movie);
        assert_eq!(MediaType::from_str("TV-Show"), MediaType::TvShow);
        assert_eq!(MediaType::from_str("GAME"), MediaType::Game);
        assert_eq!(MediaType::from_str("anime"), MediaType::Anime);
        assert_eq!(MediaType::from_str("unknown"), MediaType::Other);
    }
}
