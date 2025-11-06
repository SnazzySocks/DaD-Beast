//! Common types and newtype wrappers for the unified tracker platform.
//!
//! This module provides type-safe wrappers around primitive types to prevent
//! mixing up different kinds of identifiers and data.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// User identifier (newtype wrapper around UUID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new random user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn into_inner(self) -> Uuid {
        self.0
    }

    /// Get a reference to the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Torrent identifier (newtype wrapper around UUID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TorrentId(pub Uuid);

impl TorrentId {
    /// Create a new random torrent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn into_inner(self) -> Uuid {
        self.0
    }

    /// Get a reference to the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TorrentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TorrentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TorrentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for TorrentId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// BitTorrent info hash (20-byte SHA1 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfoHash(pub [u8; 20]);

impl InfoHash {
    /// Create an info hash from a byte slice
    ///
    /// # Errors
    ///
    /// Returns an error if the slice is not exactly 20 bytes
    pub fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 20 {
            return Err(format!(
                "Invalid info hash length: expected 20 bytes, got {}",
                bytes.len()
            ));
        }

        let mut hash = [0u8; 20];
        hash.copy_from_slice(bytes);
        Ok(Self(hash))
    }

    /// Create an info hash from a hex string
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not valid hex or not 40 characters
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        if hex.len() != 40 {
            return Err(format!(
                "Invalid info hash hex length: expected 40 characters, got {}",
                hex.len()
            ));
        }

        let mut bytes = [0u8; 20];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|e| format!("Invalid UTF-8 in hex string: {}", e))?;
            bytes[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|e| format!("Invalid hex character: {}", e))?;
        }

        Ok(Self(bytes))
    }

    /// Convert the info hash to a hex string
    pub fn to_hex(&self) -> String {
        self.0
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Serialize for InfoHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for InfoHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        Self::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

/// User passkey for tracker authentication (32-character hexadecimal string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Passkey(pub String);

impl Passkey {
    /// Generate a new random passkey
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        let hex = bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Self(hex)
    }

    /// Create a passkey from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not 32 hexadecimal characters
    pub fn from_string(s: String) -> Result<Self, String> {
        if s.len() != 32 {
            return Err(format!(
                "Invalid passkey length: expected 32 characters, got {}",
                s.len()
            ));
        }

        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Passkey must be hexadecimal".to_string());
        }

        Ok(Self(s))
    }

    /// Get the passkey as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Passkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Passkey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

/// Peer ID (20-byte identifier used by BitTorrent clients)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(pub [u8; 20]);

impl PeerId {
    /// Create a peer ID from a byte slice
    pub fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 20 {
            return Err(format!(
                "Invalid peer ID length: expected 20 bytes, got {}",
                bytes.len()
            ));
        }

        let mut id = [0u8; 20];
        id.copy_from_slice(bytes);
        Ok(Self(id))
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        )
    }
}

/// Download/Upload statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    /// Total bytes uploaded
    pub uploaded: i64,
    /// Total bytes downloaded
    pub downloaded: i64,
    /// Current ratio (uploaded / downloaded)
    pub ratio: f64,
}

impl Stats {
    /// Create new stats
    pub fn new(uploaded: i64, downloaded: i64) -> Self {
        let ratio = if downloaded > 0 {
            uploaded as f64 / downloaded as f64
        } else {
            0.0
        };

        Self {
            uploaded,
            downloaded,
            ratio,
        }
    }

    /// Create zero stats
    pub fn zero() -> Self {
        Self {
            uploaded: 0,
            downloaded: 0,
            ratio: 0.0,
        }
    }

    /// Update stats with new values
    pub fn update(&mut self, uploaded: i64, downloaded: i64) {
        self.uploaded = uploaded;
        self.downloaded = downloaded;
        self.ratio = if downloaded > 0 {
            uploaded as f64 / downloaded as f64
        } else {
            0.0
        };
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id3 = UserId::from(uuid);
        assert_eq!(id3.as_uuid(), &uuid);
    }

    #[test]
    fn test_info_hash() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        let hash = InfoHash::from_hex(hex).unwrap();
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_passkey() {
        let passkey = Passkey::generate();
        assert_eq!(passkey.as_str().len(), 32);
        assert!(passkey.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_stats() {
        let stats = Stats::new(1000, 500);
        assert_eq!(stats.ratio, 2.0);

        let stats = Stats::new(1000, 0);
        assert_eq!(stats.ratio, 0.0);
    }
}
