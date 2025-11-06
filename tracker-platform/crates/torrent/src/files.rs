//! File management for torrents
//!
//! This module handles file lists within torrents, validation, sanitization,
//! and media file detection.

use anyhow::{anyhow, Result};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// File information with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFileInfo {
    /// Sanitized file path
    pub path: String,

    /// File size in bytes
    pub size: i64,

    /// File extension (lowercase, without dot)
    pub extension: Option<String>,

    /// File type category
    pub file_type: FileType,

    /// Media type if applicable
    pub media_type: Option<MediaType>,

    /// Whether this is a sample file
    pub is_sample: bool,
}

/// File type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// Video file (mp4, mkv, avi, etc.)
    Video,

    /// Audio file (mp3, flac, wav, etc.)
    Audio,

    /// Image file (jpg, png, gif, etc.)
    Image,

    /// Document (pdf, txt, doc, etc.)
    Document,

    /// Archive (zip, rar, tar, etc.)
    Archive,

    /// Executable or binary
    Executable,

    /// NFO file (release info)
    Nfo,

    /// Subtitle file
    Subtitle,

    /// Other/unknown
    Other,
}

/// Media type for content categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Adult content
    Adult,

    /// Other
    Other,
}

/// File validation result
#[derive(Debug)]
pub struct FileValidation {
    /// Whether the file list is valid
    pub is_valid: bool,

    /// Validation errors
    pub errors: Vec<String>,

    /// Validation warnings
    pub warnings: Vec<String>,

    /// Total file count
    pub file_count: usize,

    /// Total size in bytes
    pub total_size: i64,

    /// Media files count
    pub media_file_count: usize,
}

/// Parse and validate file list from torrent
pub fn parse_file_list(
    files: Vec<crate::bencode::TorrentFile>,
    base_name: &str,
) -> Result<Vec<TorrentFileInfo>> {
    let mut result = Vec::new();

    for file in files {
        // Sanitize file path
        let sanitized_path = sanitize_file_path(&file.path)?;

        // Extract extension
        let extension = extract_extension(&sanitized_path);

        // Detect file type
        let file_type = detect_file_type(&extension);

        // Detect media type (basic heuristics)
        let media_type = detect_media_type(&file_type, &sanitized_path);

        // Check if sample file
        let is_sample = is_sample_file(&sanitized_path);

        result.push(TorrentFileInfo {
            path: sanitized_path,
            size: file.size,
            extension,
            file_type,
            media_type,
            is_sample,
        });
    }

    Ok(result)
}

/// Sanitize file path to prevent directory traversal attacks
pub fn sanitize_file_path(path: &str) -> Result<String> {
    // Convert to PathBuf for manipulation
    let path_buf = PathBuf::from(path);

    // Clean the path (removes . and .. components)
    let cleaned = path_buf.clean();

    // Convert back to string
    let cleaned_str = cleaned
        .to_str()
        .ok_or_else(|| anyhow!("Invalid UTF-8 in file path"))?;

    // Check for directory traversal attempts
    if cleaned_str.contains("..") {
        return Err(anyhow!("Invalid file path: directory traversal detected"));
    }

    // Check for absolute paths
    if cleaned.is_absolute() {
        return Err(anyhow!("Invalid file path: absolute paths not allowed"));
    }

    // Check for invalid characters (Windows-specific)
    if cleaned_str.contains('<')
        || cleaned_str.contains('>')
        || cleaned_str.contains(':')
        || cleaned_str.contains('"')
        || cleaned_str.contains('|')
        || cleaned_str.contains('?')
        || cleaned_str.contains('*')
    {
        return Err(anyhow!("Invalid file path: contains illegal characters"));
    }

    Ok(cleaned_str.to_string())
}

/// Extract file extension (lowercase, without dot)
fn extract_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Detect file type from extension
pub fn detect_file_type(extension: &Option<String>) -> FileType {
    match extension.as_deref() {
        // Video formats
        Some("mp4") | Some("mkv") | Some("avi") | Some("mov") | Some("wmv") | Some("flv")
        | Some("webm") | Some("m4v") | Some("mpg") | Some("mpeg") | Some("m2ts") | Some("ts")
        | Some("vob") | Some("3gp") | Some("ogv") => FileType::Video,

        // Audio formats
        Some("mp3") | Some("flac") | Some("wav") | Some("aac") | Some("ogg") | Some("wma")
        | Some("m4a") | Some("opus") | Some("ape") | Some("alac") | Some("aiff") | Some("dsd")
        | Some("dsf") => FileType::Audio,

        // Image formats
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp")
        | Some("svg") | Some("ico") | Some("tiff") | Some("tif") | Some("psd") | Some("raw")
        | Some("heic") | Some("heif") => FileType::Image,

        // Document formats
        Some("pdf") | Some("txt") | Some("doc") | Some("docx") | Some("odt") | Some("rtf")
        | Some("tex") | Some("md") | Some("epub") | Some("mobi") | Some("azw") | Some("azw3")
        | Some("fb2") | Some("lit") => FileType::Document,

        // Archive formats
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") | Some("bz2")
        | Some("xz") | Some("lz") | Some("lzma") | Some("tgz") | Some("tbz") | Some("txz")
        | Some("iso") | Some("dmg") => FileType::Archive,

        // Executable formats
        Some("exe") | Some("dll") | Some("so") | Some("dylib") | Some("app") | Some("apk")
        | Some("deb") | Some("rpm") | Some("msi") | Some("bin") => FileType::Executable,

        // NFO files
        Some("nfo") | Some("diz") => FileType::Nfo,

        // Subtitle formats
        Some("srt") | Some("sub") | Some("ass") | Some("ssa") | Some("vtt") | Some("idx")
        | Some("sup") => FileType::Subtitle,

        // Other/unknown
        _ => FileType::Other,
    }
}

/// Detect media type based on file type and path
fn detect_media_type(file_type: &FileType, path: &str) -> Option<MediaType> {
    let path_lower = path.to_lowercase();

    match file_type {
        FileType::Video => {
            // Check for TV show patterns
            if path_lower.contains("s0")
                || path_lower.contains("season")
                || path_lower.contains("episode")
                || path_lower.contains("e0")
            {
                Some(MediaType::TvShow)
            } else {
                Some(MediaType::Movie)
            }
        }
        FileType::Audio => Some(MediaType::Music),
        FileType::Document => {
            if path_lower.contains("book")
                || path_lower.contains("epub")
                || path_lower.contains("mobi")
            {
                Some(MediaType::Book)
            } else {
                None
            }
        }
        FileType::Executable => {
            if path_lower.contains("game") {
                Some(MediaType::Game)
            } else {
                Some(MediaType::Application)
            }
        }
        _ => None,
    }
}

/// Check if file is likely a sample
fn is_sample_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.contains("sample")
        || path_lower.contains("preview")
        || path_lower.contains("trailer")
}

/// Validate file list
pub fn validate_file_list(files: &[TorrentFileInfo]) -> FileValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check for empty file list
    if files.is_empty() {
        errors.push("File list is empty".to_string());
        return FileValidation {
            is_valid: false,
            errors,
            warnings,
            file_count: 0,
            total_size: 0,
            media_file_count: 0,
        };
    }

    let mut total_size = 0i64;
    let mut media_file_count = 0;

    for (idx, file) in files.iter().enumerate() {
        // Validate file size
        if file.size <= 0 {
            errors.push(format!("File {} has invalid size: {}", file.path, file.size));
        } else if file.size < 100 {
            warnings.push(format!(
                "File {} is very small ({} bytes)",
                file.path, file.size
            ));
        }

        total_size += file.size;

        // Count media files
        if matches!(
            file.file_type,
            FileType::Video | FileType::Audio | FileType::Image
        ) {
            media_file_count += 1;
        }

        // Warn about executable files
        if file.file_type == FileType::Executable {
            warnings.push(format!(
                "File {} is an executable - ensure this is intentional",
                file.path
            ));
        }

        // Check for duplicate file names (case-insensitive)
        for other in files.iter().skip(idx + 1) {
            if file.path.eq_ignore_ascii_case(&other.path) {
                errors.push(format!("Duplicate file name detected: {}", file.path));
            }
        }
    }

    // Validate total size
    if total_size <= 0 {
        errors.push("Total torrent size is invalid".to_string());
    }

    // Warn if no media files
    if media_file_count == 0 {
        warnings.push("No media files detected in torrent".to_string());
    }

    // Check size limits (example: 500GB max)
    const MAX_TORRENT_SIZE: i64 = 500 * 1024 * 1024 * 1024; // 500GB
    if total_size > MAX_TORRENT_SIZE {
        errors.push(format!(
            "Torrent size exceeds maximum allowed size ({}GB)",
            MAX_TORRENT_SIZE / (1024 * 1024 * 1024)
        ));
    }

    FileValidation {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        file_count: files.len(),
        total_size,
        media_file_count,
    }
}

/// Get primary media file from file list
///
/// Returns the largest media file, which is typically the main content
pub fn get_primary_media_file(files: &[TorrentFileInfo]) -> Option<&TorrentFileInfo> {
    files
        .iter()
        .filter(|f| matches!(f.file_type, FileType::Video | FileType::Audio))
        .filter(|f| !f.is_sample)
        .max_by_key(|f| f.size)
}

/// Calculate directory structure depth
pub fn calculate_path_depth(path: &str) -> usize {
    path.split('/').filter(|s| !s.is_empty()).count()
}

/// Extract file list statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct FileStatistics {
    /// Total number of files
    pub total_files: usize,

    /// Total size in bytes
    pub total_size: i64,

    /// Number of video files
    pub video_files: usize,

    /// Number of audio files
    pub audio_files: usize,

    /// Number of image files
    pub image_files: usize,

    /// Number of subtitle files
    pub subtitle_files: usize,

    /// Number of sample files
    pub sample_files: usize,

    /// Largest file size
    pub largest_file_size: i64,

    /// Average file size
    pub average_file_size: i64,
}

/// Calculate file statistics
pub fn calculate_statistics(files: &[TorrentFileInfo]) -> FileStatistics {
    let total_files = files.len();
    let total_size: i64 = files.iter().map(|f| f.size).sum();

    let video_files = files
        .iter()
        .filter(|f| f.file_type == FileType::Video)
        .count();

    let audio_files = files
        .iter()
        .filter(|f| f.file_type == FileType::Audio)
        .count();

    let image_files = files
        .iter()
        .filter(|f| f.file_type == FileType::Image)
        .count();

    let subtitle_files = files
        .iter()
        .filter(|f| f.file_type == FileType::Subtitle)
        .count();

    let sample_files = files.iter().filter(|f| f.is_sample).count();

    let largest_file_size = files.iter().map(|f| f.size).max().unwrap_or(0);

    let average_file_size = if total_files > 0 {
        total_size / total_files as i64
    } else {
        0
    };

    FileStatistics {
        total_files,
        total_size,
        video_files,
        audio_files,
        image_files,
        subtitle_files,
        sample_files,
        largest_file_size,
        average_file_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_file_path() {
        assert!(sanitize_file_path("valid/path/file.txt").is_ok());
        assert!(sanitize_file_path("../../../etc/passwd").is_err());
        assert!(sanitize_file_path("/absolute/path").is_err());
        assert!(sanitize_file_path("file<>.txt").is_err());
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            detect_file_type(&Some("mp4".to_string())),
            FileType::Video
        );
        assert_eq!(
            detect_file_type(&Some("mp3".to_string())),
            FileType::Audio
        );
        assert_eq!(detect_file_type(&Some("jpg".to_string())), FileType::Image);
        assert_eq!(detect_file_type(&Some("pdf".to_string())), FileType::Document);
        assert_eq!(detect_file_type(&Some("xyz".to_string())), FileType::Other);
    }

    #[test]
    fn test_is_sample_file() {
        assert!(is_sample_file("Movie.SAMPLE.mp4"));
        assert!(is_sample_file("preview.avi"));
        assert!(!is_sample_file("Movie.mp4"));
    }

    #[test]
    fn test_calculate_path_depth() {
        assert_eq!(calculate_path_depth("file.txt"), 1);
        assert_eq!(calculate_path_depth("dir/file.txt"), 2);
        assert_eq!(calculate_path_depth("dir/subdir/file.txt"), 3);
    }
}
