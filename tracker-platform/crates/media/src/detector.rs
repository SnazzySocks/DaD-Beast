//! Media type and information detection from torrent names
//!
//! Parses torrent names to extract:
//! - Media type (movie, TV show, game, music, anime)
//! - Title
//! - Year
//! - Season/episode for TV shows
//! - Quality/resolution
//! - Codec and audio info
//! - Release group

use crate::MediaType;
use regex::Regex;
use std::sync::OnceLock;

/// Extracted media information from torrent name
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub media_type: MediaType,
    pub title: String,
    pub year: Option<i32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub resolution: Option<String>,
    pub codec: Option<String>,
    pub audio: Option<String>,
    pub quality: Option<String>,
    pub release_group: Option<String>,
    pub platform: Option<String>,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            media_type: MediaType::Other,
            title: String::new(),
            year: None,
            season: None,
            episode: None,
            resolution: None,
            codec: None,
            audio: None,
            quality: None,
            release_group: None,
            platform: None,
        }
    }
}

/// Regex patterns for detection (compiled once)
struct Patterns {
    year: Regex,
    season_episode: Regex,
    resolution: Regex,
    codec: Regex,
    audio: Regex,
    quality: Regex,
    release_group: Regex,
    tv_patterns: Vec<Regex>,
    anime_patterns: Vec<Regex>,
    game_platforms: Vec<Regex>,
    music_patterns: Vec<Regex>,
}

static PATTERNS: OnceLock<Patterns> = OnceLock::new();

fn get_patterns() -> &'static Patterns {
    PATTERNS.get_or_init(|| Patterns {
        year: Regex::new(r"[\[\(]?(19\d{2}|20\d{2})[\]\)]?").unwrap(),
        season_episode: Regex::new(r"(?i)s(\d{1,2})e(\d{1,2})").unwrap(),
        resolution: Regex::new(r"(?i)(2160p|1080p|720p|480p|360p|4K|8K|UHD)").unwrap(),
        codec: Regex::new(r"(?i)(x264|x265|H\.?264|H\.?265|HEVC|AVC|XviD|DivX)").unwrap(),
        audio: Regex::new(r"(?i)(AAC|AC3|DTS|FLAC|MP3|Atmos|TrueHD|DDP)").unwrap(),
        quality: Regex::new(r"(?i)(BluRay|BDRip|BRRip|WEB-?DL|WEBRip|HDRip|DVDRip|HDTV)").unwrap(),
        release_group: Regex::new(r"-([A-Za-z0-9]+)$").unwrap(),
        tv_patterns: vec![
            Regex::new(r"(?i)s\d{1,2}e\d{1,2}").unwrap(),
            Regex::new(r"(?i)\d{1,2}x\d{1,2}").unwrap(),
            Regex::new(r"(?i)(season|episode|series)").unwrap(),
        ],
        anime_patterns: vec![
            Regex::new(r"(?i)\[.*\].*\d{1,3}").unwrap(),
            Regex::new(r"(?i)(anime|subbed|dubbed)").unwrap(),
        ],
        game_platforms: vec![
            Regex::new(r"(?i)(PC|Windows|Linux|MacOS)").unwrap(),
            Regex::new(r"(?i)(PS[1-5]|PlayStation)").unwrap(),
            Regex::new(r"(?i)(Xbox|X360|XboxOne|Series[XS])").unwrap(),
            Regex::new(r"(?i)(Switch|NSW|3DS|Wii)").unwrap(),
        ],
        music_patterns: vec![
            Regex::new(r"(?i)(FLAC|MP3|WAV|ALAC)").unwrap(),
            Regex::new(r"(?i)(Album|EP|Single|OST|Soundtrack)").unwrap(),
            Regex::new(r"(?i)\d{3,4}kbps").unwrap(),
        ],
    })
}

/// Detect media type and extract information from torrent name
pub fn detect_media_info(name: &str) -> MediaInfo {
    let patterns = get_patterns();
    let mut info = MediaInfo::default();

    // Extract year
    if let Some(captures) = patterns.year.captures(name) {
        if let Some(year_str) = captures.get(1) {
            info.year = year_str.as_str().parse().ok();
        }
    }

    // Extract season and episode
    if let Some(captures) = patterns.season_episode.captures(name) {
        info.season = captures.get(1).and_then(|m| m.as_str().parse().ok());
        info.episode = captures.get(2).and_then(|m| m.as_str().parse().ok());
    }

    // Extract resolution
    if let Some(captures) = patterns.resolution.captures(name) {
        info.resolution = captures.get(1).map(|m| m.as_str().to_uppercase());
    }

    // Extract codec
    if let Some(captures) = patterns.codec.captures(name) {
        info.codec = captures.get(1).map(|m| m.as_str().to_uppercase());
    }

    // Extract audio
    if let Some(captures) = patterns.audio.captures(name) {
        info.audio = captures.get(1).map(|m| m.as_str().to_uppercase());
    }

    // Extract quality
    if let Some(captures) = patterns.quality.captures(name) {
        info.quality = captures.get(1).map(|m| m.as_str().to_string());
    }

    // Extract release group
    if let Some(captures) = patterns.release_group.captures(name) {
        info.release_group = captures.get(1).map(|m| m.as_str().to_string());
    }

    // Detect media type
    info.media_type = detect_media_type(name, &info);

    // Extract title
    info.title = extract_title(name, &info);

    // Extract game platform if it's a game
    if info.media_type == MediaType::Game {
        for platform_re in &patterns.game_platforms {
            if let Some(captures) = platform_re.captures(name) {
                info.platform = captures.get(1).map(|m| m.as_str().to_string());
                break;
            }
        }
    }

    info
}

/// Detect the media type based on patterns
fn detect_media_type(name: &str, info: &MediaInfo) -> MediaType {
    let patterns = get_patterns();
    let name_lower = name.to_lowercase();

    // Check for music first (most specific patterns)
    for music_re in &patterns.music_patterns {
        if music_re.is_match(&name_lower) {
            return MediaType::Music;
        }
    }

    // Check for anime patterns
    for anime_re in &patterns.anime_patterns {
        if anime_re.is_match(name) {
            return MediaType::Anime;
        }
    }

    // Check for game platforms
    for platform_re in &patterns.game_platforms {
        if platform_re.is_match(name) {
            return MediaType::Game;
        }
    }

    // Check for TV show patterns
    if info.season.is_some() || info.episode.is_some() {
        return MediaType::TvShow;
    }

    for tv_re in &patterns.tv_patterns {
        if tv_re.is_match(&name_lower) {
            return MediaType::TvShow;
        }
    }

    // Check for explicit game indicators
    if name_lower.contains("game")
        || name_lower.contains("repack")
        || name_lower.contains("cracked")
    {
        return MediaType::Game;
    }

    // Default to movie if we have video quality indicators
    if info.resolution.is_some()
        || info.quality.is_some()
        || info.codec.is_some()
    {
        return MediaType::Movie;
    }

    MediaType::Other
}

/// Extract clean title from torrent name
fn extract_title(name: &str, info: &MediaInfo) -> String {
    let mut title = name.to_string();

    // Remove common separators
    title = title.replace('.', " ").replace('_', " ");

    // Remove year if present
    if let Some(year) = info.year {
        title = title.replace(&format!("({})", year), "");
        title = title.replace(&format!("[{}]", year), "");
        title = title.replace(&year.to_string(), "");
    }

    // Remove season/episode info
    if let (Some(season), Some(episode)) = (info.season, info.episode) {
        let se_pattern = format!("S{:02}E{:02}", season, episode);
        title = title.replace(&se_pattern, "");
        let se_pattern_lower = se_pattern.to_lowercase();
        title = title.to_lowercase().replace(&se_pattern_lower, "");
    }

    // Remove quality indicators
    let quality_indicators = [
        "2160p", "1080p", "720p", "480p", "360p", "4K", "8K", "UHD",
        "BluRay", "BDRip", "BRRip", "WEB-DL", "WEBRip", "WEBDL", "HDRip",
        "DVDRip", "HDTV", "x264", "x265", "H264", "H265", "HEVC", "AVC",
        "AAC", "AC3", "DTS", "FLAC", "MP3", "Atmos", "TrueHD", "DDP",
        "REPACK", "PROPER", "INTERNAL", "LIMITED",
    ];

    for indicator in &quality_indicators {
        title = title
            .replace(&indicator.to_lowercase(), " ")
            .replace(indicator, " ");
    }

    // Remove brackets and their content
    title = Regex::new(r"\[.*?\]").unwrap().replace_all(&title, "").to_string();
    title = Regex::new(r"\(.*?\)").unwrap().replace_all(&title, "").to_string();

    // Remove release group (usually at the end after a dash)
    if let Some(dash_pos) = title.rfind('-') {
        let possible_group = &title[dash_pos + 1..].trim();
        if possible_group.len() < 20 && !possible_group.contains(' ') {
            title = title[..dash_pos].to_string();
        }
    }

    // Clean up multiple spaces and trim
    title = Regex::new(r"\s+").unwrap().replace_all(&title, " ").trim().to_string();

    title
}

/// Parse season and episode from various formats
pub fn parse_season_episode(name: &str) -> Option<(u32, u32)> {
    let patterns = get_patterns();

    // Try S##E## format
    if let Some(captures) = patterns.season_episode.captures(name) {
        let season = captures.get(1)?.as_str().parse().ok()?;
        let episode = captures.get(2)?.as_str().parse().ok()?;
        return Some((season, episode));
    }

    // Try ##x## format
    let alt_pattern = Regex::new(r"(?i)(\d{1,2})x(\d{1,2})").unwrap();
    if let Some(captures) = alt_pattern.captures(name) {
        let season = captures.get(1)?.as_str().parse().ok()?;
        let episode = captures.get(2)?.as_str().parse().ok()?;
        return Some((season, episode));
    }

    None
}

/// Detect if name represents a season pack
pub fn is_season_pack(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.contains("season")
        || name_lower.contains("complete")
        || Regex::new(r"(?i)s\d{1,2}\s*(complete|full)")
            .unwrap()
            .is_match(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movie_detection() {
        let info = detect_media_info("The.Matrix.1999.1080p.BluRay.x264-GROUP");
        assert_eq!(info.media_type, MediaType::Movie);
        assert_eq!(info.title, "The Matrix");
        assert_eq!(info.year, Some(1999));
        assert_eq!(info.resolution, Some("1080P".to_string()));
    }

    #[test]
    fn test_tv_show_detection() {
        let info = detect_media_info("Breaking.Bad.S01E01.720p.WEB-DL.x264");
        assert_eq!(info.media_type, MediaType::TvShow);
        assert_eq!(info.title, "Breaking Bad");
        assert_eq!(info.season, Some(1));
        assert_eq!(info.episode, Some(1));
        assert_eq!(info.resolution, Some("720P".to_string()));
    }

    #[test]
    fn test_anime_detection() {
        let info = detect_media_info("[SubGroup] Anime Title - 12 [1080p]");
        assert_eq!(info.media_type, MediaType::Anime);
    }

    #[test]
    fn test_game_detection() {
        let info = detect_media_info("Game.Title.2023.PC-REPACK");
        assert_eq!(info.media_type, MediaType::Game);
        assert_eq!(info.platform, Some("PC".to_string()));
    }

    #[test]
    fn test_music_detection() {
        let info = detect_media_info("Artist - Album (2023) [FLAC]");
        assert_eq!(info.media_type, MediaType::Music);
        assert_eq!(info.year, Some(2023));
    }

    #[test]
    fn test_season_episode_parsing() {
        assert_eq!(parse_season_episode("S02E13"), Some((2, 13)));
        assert_eq!(parse_season_episode("2x13"), Some((2, 13)));
        assert_eq!(parse_season_episode("s05e09"), Some((5, 9)));
    }

    #[test]
    fn test_season_pack_detection() {
        assert!(is_season_pack("Show.S01.Complete"));
        assert!(is_season_pack("Show.Season.1.1080p"));
        assert!(!is_season_pack("Show.S01E01"));
    }

    #[test]
    fn test_title_extraction() {
        let info = detect_media_info("The.Lord.of.the.Rings.2001.Extended.1080p.BluRay-GROUP");
        assert!(info.title.contains("Lord"));
    }
}
