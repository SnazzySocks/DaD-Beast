//! Torrent metadata management
//!
//! This module handles torrent metadata including names, descriptions, categories,
//! tags, media type detection, quality indicators, external ID linking, NFO parsing,
//! and media storage.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::Validate;

/// Torrent metadata structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TorrentMetadata {
    /// Torrent UUID
    pub id: Uuid,

    /// Display name (can differ from .torrent file name)
    #[validate(length(min = 3, max = 255))]
    pub name: String,

    /// Detailed description (supports Markdown)
    #[validate(length(max = 50000))]
    pub description: Option<String>,

    /// Category ID
    pub category_id: Uuid,

    /// Media type
    pub media_type: MediaType,

    /// Content tags (user-votable)
    pub tags: Vec<String>,

    /// Quality information
    pub quality: Option<QualityInfo>,

    /// External database IDs
    pub external_ids: ExternalIds,

    /// NFO file content
    pub nfo_content: Option<String>,

    /// Media URLs (screenshots, posters, etc.)
    pub media_urls: MediaUrls,

    /// Release year
    pub year: Option<i32>,

    /// IMDB rating (cached)
    pub imdb_rating: Option<f32>,

    /// Is featured/highlighted
    pub is_featured: bool,

    /// Is sticky (pinned to top)
    pub is_sticky: bool,
}

/// Media type for content categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_type")]
pub enum MediaType {
    /// Movie/Film
    Movie,

    /// TV Show/Series
    TvShow,

    /// Music/Album
    Music,

    /// Game/Software
    Game,

    /// Book/Ebook
    Book,

    /// Application/Software
    Application,

    /// Adult content (18+)
    Adult,

    /// Learning materials
    Educational,

    /// Anime
    Anime,

    /// Other
    Other,
}

/// Quality indicators for media content
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "jsonb")]
pub struct QualityInfo {
    /// Video resolution (e.g., "1080p", "2160p", "720p")
    pub resolution: Option<String>,

    /// Video codec (e.g., "H.264", "H.265", "VP9", "AV1")
    pub video_codec: Option<String>,

    /// Audio codec (e.g., "AAC", "DTS", "Dolby Atmos", "FLAC")
    pub audio_codec: Option<String>,

    /// Source (e.g., "BluRay", "WEB-DL", "HDTV", "DVD")
    pub source: Option<String>,

    /// Container format (e.g., "MKV", "MP4", "AVI")
    pub container: Option<String>,

    /// Audio channels (e.g., "2.0", "5.1", "7.1")
    pub audio_channels: Option<String>,

    /// Bitrate in kbps
    pub bitrate: Option<i32>,

    /// HDR information (e.g., "HDR10", "Dolby Vision", "HDR10+")
    pub hdr: Option<String>,

    /// Release group
    pub release_group: Option<String>,

    /// Is scene release
    pub is_scene: bool,
}

/// External database IDs for linking
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "jsonb")]
pub struct ExternalIds {
    /// The Movie Database (TMDB) ID
    pub tmdb_id: Option<i64>,

    /// Internet Movie Database (IMDB) ID
    pub imdb_id: Option<String>,

    /// TheTVDB ID
    pub tvdb_id: Option<i64>,

    /// Internet Game Database (IGDB) ID
    pub igdb_id: Option<i64>,

    /// MusicBrainz ID (for music)
    pub musicbrainz_id: Option<String>,

    /// AniList ID (for anime)
    pub anilist_id: Option<i64>,

    /// MyAnimeList ID
    pub mal_id: Option<i64>,
}

/// Media URLs for screenshots, posters, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "jsonb")]
pub struct MediaUrls {
    /// Poster/cover image URL
    pub poster_url: Option<String>,

    /// Banner image URL
    pub banner_url: Option<String>,

    /// Screenshot URLs
    pub screenshots: Vec<String>,

    /// Trailer URL
    pub trailer_url: Option<String>,
}

/// Tag with voting information (Gazelle pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag UUID
    pub id: Uuid,

    /// Tag name
    pub name: String,

    /// Number of upvotes
    pub upvotes: i32,

    /// Number of downvotes
    pub downvotes: i32,

    /// Net score (upvotes - downvotes)
    pub score: i32,

    /// Timestamp when added
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Tag vote
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagVote {
    /// Upvote (+1)
    Up,

    /// Downvote (-1)
    Down,
}

/// Category for torrent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Category UUID
    pub id: Uuid,

    /// Category name
    pub name: String,

    /// Category slug (URL-friendly)
    pub slug: String,

    /// Icon/emoji
    pub icon: Option<String>,

    /// Sort order
    pub sort_order: i32,

    /// Parent category (for hierarchical structure)
    pub parent_id: Option<Uuid>,
}

/// Parse quality information from torrent name
///
/// Uses common release naming conventions to extract quality metadata
pub fn parse_quality_from_name(name: &str) -> QualityInfo {
    let name_upper = name.to_uppercase();

    // Extract resolution
    let resolution = if name_upper.contains("2160P") || name_upper.contains("4K") {
        Some("2160p".to_string())
    } else if name_upper.contains("1080P") {
        Some("1080p".to_string())
    } else if name_upper.contains("720P") {
        Some("720p".to_string())
    } else if name_upper.contains("480P") {
        Some("480p".to_string())
    } else if name_upper.contains("360P") {
        Some("360p".to_string())
    } else {
        None
    };

    // Extract video codec
    let video_codec = if name_upper.contains("H.265")
        || name_upper.contains("H265")
        || name_upper.contains("HEVC")
    {
        Some("H.265".to_string())
    } else if name_upper.contains("H.264")
        || name_upper.contains("H264")
        || name_upper.contains("X264")
    {
        Some("H.264".to_string())
    } else if name_upper.contains("AV1") {
        Some("AV1".to_string())
    } else if name_upper.contains("VP9") {
        Some("VP9".to_string())
    } else if name_upper.contains("XVID") {
        Some("XviD".to_string())
    } else {
        None
    };

    // Extract audio codec
    let audio_codec = if name_upper.contains("ATMOS") {
        Some("Dolby Atmos".to_string())
    } else if name_upper.contains("DTS-HD.MA") || name_upper.contains("DTS-HD MA") {
        Some("DTS-HD MA".to_string())
    } else if name_upper.contains("DTS") {
        Some("DTS".to_string())
    } else if name_upper.contains("TRUEHD") {
        Some("TrueHD".to_string())
    } else if name_upper.contains("FLAC") {
        Some("FLAC".to_string())
    } else if name_upper.contains("AAC") {
        Some("AAC".to_string())
    } else if name_upper.contains("AC3") {
        Some("AC3".to_string())
    } else if name_upper.contains("MP3") {
        Some("MP3".to_string())
    } else {
        None
    };

    // Extract source
    let source = if name_upper.contains("BLURAY") || name_upper.contains("BLU-RAY") {
        Some("BluRay".to_string())
    } else if name_upper.contains("WEB-DL") || name_upper.contains("WEBDL") {
        Some("WEB-DL".to_string())
    } else if name_upper.contains("WEBRIP") {
        Some("WEBRip".to_string())
    } else if name_upper.contains("HDTV") {
        Some("HDTV".to_string())
    } else if name_upper.contains("DVDRIP") {
        Some("DVDRip".to_string())
    } else if name_upper.contains("DVD") {
        Some("DVD".to_string())
    } else if name_upper.contains("BDRIP") {
        Some("BDRip".to_string())
    } else if name_upper.contains("CAM") {
        Some("CAM".to_string())
    } else if name_upper.contains("TS") || name_upper.contains("TELESYNC") {
        Some("Telesync".to_string())
    } else {
        None
    };

    // Extract container
    let container = if name.ends_with(".mkv") || name_upper.contains("MKV") {
        Some("MKV".to_string())
    } else if name.ends_with(".mp4") || name_upper.contains("MP4") {
        Some("MP4".to_string())
    } else if name.ends_with(".avi") || name_upper.contains("AVI") {
        Some("AVI".to_string())
    } else {
        None
    };

    // Extract audio channels
    let audio_channels = if name_upper.contains("7.1") {
        Some("7.1".to_string())
    } else if name_upper.contains("5.1") {
        Some("5.1".to_string())
    } else if name_upper.contains("2.0") {
        Some("2.0".to_string())
    } else {
        None
    };

    // Extract HDR info
    let hdr = if name_upper.contains("DOLBY VISION") || name_upper.contains("DV") {
        Some("Dolby Vision".to_string())
    } else if name_upper.contains("HDR10+") {
        Some("HDR10+".to_string())
    } else if name_upper.contains("HDR10") || name_upper.contains("HDR") {
        Some("HDR10".to_string())
    } else {
        None
    };

    // Extract release group (typically after last '-')
    let release_group = name
        .rsplit('-')
        .next()
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '.'))
        .map(|s| s.to_string());

    // Check if scene release
    let is_scene = name_upper.contains("-SCENE") || name_upper.contains("[SCENE]");

    QualityInfo {
        resolution,
        video_codec,
        audio_codec,
        source,
        container,
        audio_channels,
        bitrate: None,
        hdr,
        release_group,
        is_scene,
    }
}

/// Parse NFO file content
///
/// NFO files contain release information in ASCII art format
pub fn parse_nfo(content: &[u8]) -> Result<String> {
    // Try UTF-8 first
    if let Ok(text) = std::str::from_utf8(content) {
        return Ok(text.to_string());
    }

    // Try Windows-1252 (CP-1252) encoding for legacy NFO files
    let text = content
        .iter()
        .map(|&b| b as char)
        .collect::<String>();

    if text.is_empty() {
        return Err(anyhow!("NFO file is empty"));
    }

    Ok(text)
}

/// Extract media type from category and file analysis
pub fn determine_media_type(
    category_name: &str,
    file_stats: &crate::files::FileStatistics,
) -> MediaType {
    let category_lower = category_name.to_lowercase();

    if category_lower.contains("movie") || category_lower.contains("film") {
        MediaType::Movie
    } else if category_lower.contains("tv")
        || category_lower.contains("series")
        || category_lower.contains("show")
    {
        MediaType::TvShow
    } else if category_lower.contains("music") || category_lower.contains("audio") {
        MediaType::Music
    } else if category_lower.contains("game") {
        MediaType::Game
    } else if category_lower.contains("book")
        || category_lower.contains("ebook")
        || category_lower.contains("audiobook")
    {
        MediaType::Book
    } else if category_lower.contains("software")
        || category_lower.contains("app")
        || category_lower.contains("application")
    {
        MediaType::Application
    } else if category_lower.contains("anime") {
        MediaType::Anime
    } else if category_lower.contains("education")
        || category_lower.contains("learning")
        || category_lower.contains("course")
    {
        MediaType::Educational
    } else if category_lower.contains("adult") || category_lower.contains("xxx") {
        MediaType::Adult
    } else {
        // Fallback to file-based detection
        if file_stats.video_files > 0 {
            MediaType::Movie
        } else if file_stats.audio_files > 0 {
            MediaType::Music
        } else {
            MediaType::Other
        }
    }
}

/// Database operations for metadata
impl TorrentMetadata {
    /// Insert torrent metadata into database
    pub async fn insert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO torrent_metadata (
                id, name, description, category_id, media_type,
                tags, quality, external_ids, nfo_content, media_urls,
                year, imdb_rating, is_featured, is_sticky
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            self.id,
            self.name,
            self.description,
            self.category_id,
            self.media_type as MediaType,
            &self.tags,
            serde_json::to_value(&self.quality)?,
            serde_json::to_value(&self.external_ids)?,
            self.nfo_content,
            serde_json::to_value(&self.media_urls)?,
            self.year,
            self.imdb_rating,
            self.is_featured,
            self.is_sticky,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update torrent metadata
    pub async fn update(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrent_metadata
            SET name = $2, description = $3, category_id = $4, media_type = $5,
                tags = $6, quality = $7, external_ids = $8, nfo_content = $9,
                media_urls = $10, year = $11, imdb_rating = $12,
                is_featured = $13, is_sticky = $14
            WHERE id = $1
            "#,
            self.id,
            self.name,
            self.description,
            self.category_id,
            self.media_type as MediaType,
            &self.tags,
            serde_json::to_value(&self.quality)?,
            serde_json::to_value(&self.external_ids)?,
            self.nfo_content,
            serde_json::to_value(&self.media_urls)?,
            self.year,
            self.imdb_rating,
            self.is_featured,
            self.is_sticky,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get metadata by torrent ID
    pub async fn get_by_id(id: Uuid, pool: &PgPool) -> Result<Option<Self>> {
        let record = sqlx::query!(
            r#"
            SELECT id, name, description, category_id, media_type as "media_type: MediaType",
                   tags, quality, external_ids, nfo_content, media_urls,
                   year, imdb_rating, is_featured, is_sticky
            FROM torrent_metadata
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        match record {
            Some(r) => Ok(Some(Self {
                id: r.id,
                name: r.name,
                description: r.description,
                category_id: r.category_id,
                media_type: r.media_type,
                tags: r.tags,
                quality: r.quality.and_then(|v| serde_json::from_value(v).ok()),
                external_ids: r
                    .external_ids
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                nfo_content: r.nfo_content,
                media_urls: r
                    .media_urls
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                year: r.year,
                imdb_rating: r.imdb_rating,
                is_featured: r.is_featured,
                is_sticky: r.is_sticky,
            })),
            None => Ok(None),
        }
    }
}

/// Tag operations
pub struct TagService;

impl TagService {
    /// Add or vote on a tag
    pub async fn vote_tag(
        torrent_id: Uuid,
        user_id: Uuid,
        tag_name: &str,
        vote: TagVote,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<()> {
        // Normalize tag name (lowercase, trim)
        let normalized_tag = tag_name.trim().to_lowercase();

        // Insert or update vote
        let vote_value = match vote {
            TagVote::Up => 1,
            TagVote::Down => -1,
        };

        sqlx::query!(
            r#"
            INSERT INTO torrent_tags (torrent_id, tag_name, user_id, vote)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (torrent_id, tag_name, user_id)
            DO UPDATE SET vote = $4, updated_at = NOW()
            "#,
            torrent_id,
            normalized_tag,
            user_id,
            vote_value,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Get tags for a torrent with scores
    pub async fn get_tags(torrent_id: Uuid, pool: &PgPool) -> Result<Vec<Tag>> {
        let records = sqlx::query!(
            r#"
            SELECT
                gen_random_uuid() as id,
                tag_name as name,
                SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END)::int as upvotes,
                SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END)::int as downvotes,
                SUM(vote)::int as score,
                MIN(created_at) as created_at
            FROM torrent_tags
            WHERE torrent_id = $1
            GROUP BY tag_name
            HAVING SUM(vote) > -5
            ORDER BY SUM(vote) DESC, COUNT(*) DESC
            LIMIT 50
            "#,
            torrent_id
        )
        .fetch_all(pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| Tag {
                id: r.id.unwrap(),
                name: r.name,
                upvotes: r.upvotes.unwrap_or(0),
                downvotes: r.downvotes.unwrap_or(0),
                score: r.score.unwrap_or(0),
                created_at: r.created_at.unwrap(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quality_from_name() {
        let quality = parse_quality_from_name(
            "Movie.Name.2023.2160p.BluRay.x265.HDR10.DTS-HD.MA.7.1-GROUPNAME",
        );

        assert_eq!(quality.resolution, Some("2160p".to_string()));
        assert_eq!(quality.video_codec, Some("H.265".to_string()));
        assert_eq!(quality.audio_codec, Some("DTS-HD MA".to_string()));
        assert_eq!(quality.source, Some("BluRay".to_string()));
        assert_eq!(quality.audio_channels, Some("7.1".to_string()));
        assert_eq!(quality.hdr, Some("HDR10".to_string()));
        assert_eq!(quality.release_group, Some("GROUPNAME".to_string()));
    }

    #[test]
    fn test_determine_media_type() {
        use crate::files::FileStatistics;

        let video_stats = FileStatistics {
            total_files: 10,
            total_size: 5000000000,
            video_files: 5,
            audio_files: 0,
            image_files: 0,
            subtitle_files: 2,
            sample_files: 0,
            largest_file_size: 3000000000,
            average_file_size: 500000000,
        };

        assert_eq!(
            determine_media_type("Movies", &video_stats),
            MediaType::Movie
        );
        assert_eq!(
            determine_media_type("TV Shows", &video_stats),
            MediaType::TvShow
        );
    }
}
