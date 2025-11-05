//! Bencode parsing utilities for .torrent files
//!
//! This module provides comprehensive support for parsing BitTorrent metainfo files
//! (`.torrent` files) which use Bencode encoding. It supports both BitTorrent v1
//! and v2 specifications.
//!
//! # Bencode Format
//!
//! Bencode supports four data types:
//! - Strings: `<length>:<contents>` (e.g., `4:spam`)
//! - Integers: `i<number>e` (e.g., `i42e`)
//! - Lists: `l<elements>e` (e.g., `l4:spam4:eggse`)
//! - Dictionaries: `d<key><value>...e` (e.g., `d3:cow3:moo4:spam4:eggse`)

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;

/// BitTorrent metainfo file structure (v1 and v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    /// Announce URL for the tracker
    pub announce: Option<String>,

    /// Announce URL list for multi-tracker support
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,

    /// Comment about the torrent
    pub comment: Option<String>,

    /// Creation date (Unix timestamp)
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,

    /// Creator tool/program
    #[serde(rename = "created by")]
    pub created_by: Option<String>,

    /// Info dictionary (contains file/piece information)
    pub info: Info,

    /// Encoding used for the torrent (usually UTF-8)
    pub encoding: Option<String>,

    /// Web seeds for HTTP/FTP seeding
    #[serde(rename = "url-list")]
    pub url_list: Option<Vec<String>>,

    /// DHT nodes for trackerless torrents
    pub nodes: Option<Vec<(String, i64)>>,
}

/// Info dictionary structure (the part that's hashed for info_hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    /// Piece length in bytes
    #[serde(rename = "piece length")]
    pub piece_length: i64,

    /// Concatenated SHA1 hashes of all pieces
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,

    /// Private flag (1 = private torrent, disable DHT/PEX)
    pub private: Option<i64>,

    /// Name of the file or directory
    pub name: String,

    /// For single-file torrents: file length in bytes
    pub length: Option<i64>,

    /// For single-file torrents: MD5 checksum (deprecated)
    pub md5sum: Option<String>,

    /// For multi-file torrents: list of files
    pub files: Option<Vec<FileInfo>>,

    /// BitTorrent v2: file tree structure
    #[serde(rename = "file tree")]
    pub file_tree: Option<BTreeMap<String, serde_bencode::value::Value>>,

    /// BitTorrent v2: pieces root hash
    #[serde(rename = "pieces root")]
    #[serde(with = "serde_bytes")]
    pub pieces_root: Option<Vec<u8>>,

    /// BitTorrent v2: meta version
    #[serde(rename = "meta version")]
    pub meta_version: Option<i64>,
}

/// File information for multi-file torrents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File length in bytes
    pub length: i64,

    /// File path components (directory/file hierarchy)
    pub path: Vec<String>,

    /// MD5 checksum (deprecated)
    pub md5sum: Option<String>,

    /// File attributes (BitTorrent v2)
    pub attr: Option<String>,
}

/// Parsed torrent information with calculated values
#[derive(Debug, Clone)]
pub struct TorrentInfo {
    /// Original torrent structure
    pub torrent: Torrent,

    /// Calculated info_hash (SHA1 of info dictionary)
    pub info_hash: String,

    /// Raw info_hash bytes
    pub info_hash_bytes: [u8; 20],

    /// Total size in bytes
    pub total_size: i64,

    /// Number of pieces
    pub piece_count: usize,

    /// List of files with sizes
    pub file_list: Vec<TorrentFile>,

    /// Whether this is a multi-file torrent
    pub is_multi_file: bool,

    /// Announce URLs (deduplicated and flattened)
    pub announce_urls: Vec<String>,

    /// Whether this is a private torrent
    pub is_private: bool,

    /// BitTorrent version (1 or 2)
    pub version: u8,
}

/// Simplified file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    /// Full file path
    pub path: String,

    /// File size in bytes
    pub size: i64,

    /// File offset in the torrent (for piece calculation)
    pub offset: i64,
}

impl Torrent {
    /// Parse a .torrent file from bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes of the .torrent file
    ///
    /// # Returns
    ///
    /// A parsed `Torrent` structure
    ///
    /// # Errors
    ///
    /// Returns an error if the bencode is malformed
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_bencode::from_bytes(data)
            .context("Failed to parse torrent file: invalid bencode format")
    }

    /// Parse a .torrent file and calculate info_hash
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes of the .torrent file
    ///
    /// # Returns
    ///
    /// A `TorrentInfo` structure with parsed data and calculated values
    pub fn parse(data: &[u8]) -> Result<TorrentInfo> {
        let torrent = Self::from_bytes(data)?;

        // Calculate info_hash
        let info_hash_bytes = calculate_info_hash(data)?;
        let info_hash = hex::encode(info_hash_bytes);

        // Extract file list and calculate total size
        let (file_list, total_size) = extract_file_list(&torrent.info)?;

        // Calculate piece count
        let piece_count = torrent.info.pieces.len() / 20;

        // Extract and deduplicate announce URLs
        let announce_urls = extract_announce_urls(&torrent);

        // Check if private
        let is_private = torrent.info.private.unwrap_or(0) != 0;

        // Determine version
        let version = if torrent.info.meta_version.is_some() || torrent.info.file_tree.is_some() {
            2
        } else {
            1
        };

        Ok(TorrentInfo {
            torrent,
            info_hash,
            info_hash_bytes,
            total_size,
            piece_count,
            file_list,
            is_multi_file: file_list.len() > 1,
            announce_urls,
            is_private,
            version,
        })
    }
}

impl Info {
    /// Validate the info dictionary structure
    pub fn validate(&self) -> Result<()> {
        // Validate piece length (must be power of 2, typically 16KB to 16MB)
        if self.piece_length <= 0 {
            return Err(anyhow!("Invalid piece length: must be positive"));
        }

        if !is_power_of_two(self.piece_length as u64) {
            return Err(anyhow!("Invalid piece length: must be a power of 2"));
        }

        if self.piece_length < 16384 || self.piece_length > 16777216 {
            return Err(anyhow!("Invalid piece length: must be between 16KB and 16MB"));
        }

        // Validate pieces (must be multiple of 20 bytes for SHA1 hashes)
        if self.pieces.len() % 20 != 0 {
            return Err(anyhow!("Invalid pieces: length must be multiple of 20"));
        }

        if self.pieces.is_empty() {
            return Err(anyhow!("Invalid pieces: cannot be empty"));
        }

        // Validate name
        if self.name.is_empty() {
            return Err(anyhow!("Invalid name: cannot be empty"));
        }

        // Validate file structure (must have either length or files, not both)
        match (&self.length, &self.files) {
            (Some(length), None) => {
                if *length <= 0 {
                    return Err(anyhow!("Invalid file length: must be positive"));
                }
            }
            (None, Some(files)) => {
                if files.is_empty() {
                    return Err(anyhow!("Invalid files: list cannot be empty"));
                }
                for file in files {
                    if file.length <= 0 {
                        return Err(anyhow!("Invalid file length: must be positive"));
                    }
                    if file.path.is_empty() {
                        return Err(anyhow!("Invalid file path: cannot be empty"));
                    }
                }
            }
            (Some(_), Some(_)) => {
                return Err(anyhow!("Invalid torrent: cannot have both 'length' and 'files'"));
            }
            (None, None) => {
                // Check for v2 file tree
                if self.file_tree.is_none() {
                    return Err(anyhow!("Invalid torrent: must have 'length', 'files', or 'file tree'"));
                }
            }
        }

        Ok(())
    }
}

/// Calculate the info_hash from raw torrent bytes
///
/// The info_hash is the SHA1 hash of the bencoded info dictionary.
/// This is the unique identifier for a torrent.
fn calculate_info_hash(data: &[u8]) -> Result<[u8; 20]> {
    // Find the info dictionary in the bencode data
    let info_start = find_info_dict_start(data)?;
    let info_end = find_info_dict_end(data, info_start)?;

    // Extract the info dictionary bytes
    let info_bytes = &data[info_start..info_end];

    // Calculate SHA1 hash
    let mut hasher = Sha1::new();
    hasher.update(info_bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(hash)
}

/// Find the start position of the info dictionary in bencode data
fn find_info_dict_start(data: &[u8]) -> Result<usize> {
    // Look for "4:info" in the bencode data
    let pattern = b"4:info";

    for (i, window) in data.windows(pattern.len()).enumerate() {
        if window == pattern {
            // Return position after "4:info"
            return Ok(i + pattern.len());
        }
    }

    Err(anyhow!("Info dictionary not found in torrent file"))
}

/// Find the end position of the info dictionary in bencode data
fn find_info_dict_end(data: &[u8], start: usize) -> Result<usize> {
    let mut depth = 0;
    let mut i = start;

    while i < data.len() {
        match data[i] {
            b'd' => depth += 1, // Dictionary start
            b'l' => depth += 1, // List start
            b'e' => {
                if depth == 0 {
                    return Ok(i + 1); // Include the 'e'
                }
                depth -= 1;
            }
            b'i' => {
                // Skip integer
                i += 1;
                while i < data.len() && data[i] != b'e' {
                    i += 1;
                }
            }
            b'0'..=b'9' => {
                // Skip string
                let mut len_str = Vec::new();
                while i < data.len() && data[i] != b':' {
                    len_str.push(data[i]);
                    i += 1;
                }
                if i >= data.len() {
                    return Err(anyhow!("Unexpected end of data while parsing string length"));
                }
                i += 1; // Skip ':'

                let len = String::from_utf8_lossy(&len_str)
                    .parse::<usize>()
                    .context("Invalid string length in bencode")?;
                i += len - 1; // Will be incremented at end of loop
            }
            _ => {}
        }
        i += 1;
    }

    Err(anyhow!("Could not find end of info dictionary"))
}

/// Extract file list from info dictionary
fn extract_file_list(info: &Info) -> Result<(Vec<TorrentFile>, i64)> {
    let mut files = Vec::new();
    let mut offset = 0i64;

    if let Some(length) = info.length {
        // Single file torrent
        files.push(TorrentFile {
            path: info.name.clone(),
            size: length,
            offset: 0,
        });
        Ok((files, length))
    } else if let Some(file_list) = &info.files {
        // Multi-file torrent
        let mut total_size = 0i64;

        for file in file_list {
            let path = format!("{}/{}", info.name, file.path.join("/"));
            files.push(TorrentFile {
                path,
                size: file.length,
                offset,
            });
            offset += file.length;
            total_size += file.length;
        }

        Ok((files, total_size))
    } else {
        // BitTorrent v2 with file tree
        Err(anyhow!("BitTorrent v2 file tree parsing not yet fully implemented"))
    }
}

/// Extract and deduplicate announce URLs
fn extract_announce_urls(torrent: &Torrent) -> Vec<String> {
    let mut urls = Vec::new();

    // Add main announce URL
    if let Some(ref announce) = torrent.announce {
        urls.push(announce.clone());
    }

    // Add announce-list URLs
    if let Some(ref announce_list) = torrent.announce_list {
        for tier in announce_list {
            for url in tier {
                if !urls.contains(url) {
                    urls.push(url.clone());
                }
            }
        }
    }

    urls
}

/// Check if a number is a power of two
fn is_power_of_two(n: u64) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Validate announce URL
///
/// Ensures the announce URL is valid and uses an allowed protocol
pub fn validate_announce_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url)
        .context("Invalid announce URL format")?;

    match parsed.scheme() {
        "http" | "https" | "udp" => Ok(()),
        _ => Err(anyhow!("Invalid announce URL protocol: must be http, https, or udp")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(16384));
        assert!(is_power_of_two(262144));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(100));
    }

    #[test]
    fn test_validate_announce_url() {
        assert!(validate_announce_url("http://tracker.example.com:8080/announce").is_ok());
        assert!(validate_announce_url("https://tracker.example.com/announce").is_ok());
        assert!(validate_announce_url("udp://tracker.example.com:6969").is_ok());
        assert!(validate_announce_url("ftp://tracker.example.com").is_err());
        assert!(validate_announce_url("not-a-url").is_err());
    }
}
