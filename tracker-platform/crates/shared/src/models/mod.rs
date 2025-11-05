//! Database models for the unified tracker platform.
//!
//! This module contains the core database models used across the application.
//! All models derive from SQLx traits for seamless database integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::types::{InfoHash, Passkey, Stats, TorrentId, UserId};

/// User model representing a registered user in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    /// Username (unique)
    pub username: String,
    /// Email address (unique)
    pub email: String,
    /// Hashed password
    pub password_hash: String,
    /// User's passkey for tracker authentication
    pub passkey: String,
    /// Total bytes uploaded
    pub uploaded: i64,
    /// Total bytes downloaded
    pub downloaded: i64,
    /// User role (user, moderator, admin)
    pub role: String,
    /// Account status (active, suspended, banned)
    pub status: String,
    /// Whether email is verified
    pub email_verified: bool,
    /// Two-factor authentication enabled
    pub two_factor_enabled: bool,
    /// Two-factor authentication secret (encrypted)
    #[serde(skip_serializing)]
    pub two_factor_secret: Option<String>,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Last login timestamp
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    /// Get user ID as UserId type
    pub fn user_id(&self) -> UserId {
        UserId(self.id)
    }

    /// Get passkey as Passkey type
    pub fn passkey(&self) -> Result<Passkey, String> {
        Passkey::from_string(self.passkey.clone())
    }

    /// Get user stats
    pub fn stats(&self) -> Stats {
        Stats::new(self.uploaded, self.downloaded)
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("admin")
    }

    /// Check if user is moderator or admin
    pub fn is_moderator(&self) -> bool {
        self.is_admin() || self.role.eq_ignore_ascii_case("moderator")
    }

    /// Check if user account is active
    pub fn is_active(&self) -> bool {
        self.status.eq_ignore_ascii_case("active")
    }

    /// Check if user can download
    pub fn can_download(&self) -> bool {
        self.is_active() && self.email_verified
    }
}

/// Torrent model representing a torrent file in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Torrent {
    /// Unique torrent identifier
    pub id: Uuid,
    /// BitTorrent info hash (hex string)
    pub info_hash: String,
    /// Torrent name
    pub name: String,
    /// Total size in bytes
    pub size: i64,
    /// Number of files
    pub file_count: i32,
    /// Category
    pub category: String,
    /// Description
    pub description: Option<String>,
    /// User who uploaded the torrent
    pub uploader_id: Uuid,
    /// Number of seeders
    pub seeders: i32,
    /// Number of leechers
    pub leechers: i32,
    /// Number of completed downloads
    pub completed: i32,
    /// Torrent status (pending, approved, rejected)
    pub status: String,
    /// Whether torrent is marked as featured
    pub featured: bool,
    /// Upload timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Torrent {
    /// Get torrent ID as TorrentId type
    pub fn torrent_id(&self) -> TorrentId {
        TorrentId(self.id)
    }

    /// Get info hash as InfoHash type
    pub fn info_hash(&self) -> Result<InfoHash, String> {
        InfoHash::from_hex(&self.info_hash)
    }

    /// Get uploader ID as UserId type
    pub fn uploader_id(&self) -> UserId {
        UserId(self.uploader_id)
    }

    /// Check if torrent is approved
    pub fn is_approved(&self) -> bool {
        self.status.eq_ignore_ascii_case("approved")
    }

    /// Check if torrent has seeders
    pub fn has_seeders(&self) -> bool {
        self.seeders > 0
    }

    /// Get total peers (seeders + leechers)
    pub fn total_peers(&self) -> i32 {
        self.seeders + self.leechers
    }
}

/// Peer model representing a peer connection to a torrent
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Peer {
    /// Unique peer identifier
    pub id: Uuid,
    /// Associated torrent
    pub torrent_id: Uuid,
    /// Associated user (via passkey)
    pub user_id: Uuid,
    /// Peer ID from BitTorrent client
    pub peer_id: String,
    /// IP address
    pub ip_address: String,
    /// Port number
    pub port: i32,
    /// Bytes uploaded in this session
    pub uploaded: i64,
    /// Bytes downloaded in this session
    pub downloaded: i64,
    /// Bytes left to download
    pub left: i64,
    /// Peer state (seeder, leecher, stopped)
    pub state: String,
    /// User agent string
    pub user_agent: Option<String>,
    /// First announce timestamp
    pub created_at: DateTime<Utc>,
    /// Last announce timestamp
    pub last_announce_at: DateTime<Utc>,
}

impl Peer {
    /// Get torrent ID as TorrentId type
    pub fn torrent_id(&self) -> TorrentId {
        TorrentId(self.torrent_id)
    }

    /// Get user ID as UserId type
    pub fn user_id(&self) -> UserId {
        UserId(self.user_id)
    }

    /// Check if peer is seeder
    pub fn is_seeder(&self) -> bool {
        self.left == 0
    }

    /// Check if peer is leecher
    pub fn is_leecher(&self) -> bool {
        self.left > 0
    }

    /// Check if peer is active (announced in last 3 hours)
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_announce_at);
        duration.num_hours() < 3
    }
}

/// Trait for models with timestamps
pub trait Timestamped {
    /// Get creation timestamp
    fn created_at(&self) -> DateTime<Utc>;

    /// Get last update timestamp
    fn updated_at(&self) -> DateTime<Utc>;
}

impl Timestamped for User {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Timestamped for Torrent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Timestamped for Peer {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.last_announce_at
    }
}

/// Trait for models that can be identified by UUID
pub trait Identifiable {
    /// Get the model's UUID
    fn id(&self) -> Uuid;
}

impl Identifiable for User {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for Torrent {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for Peer {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_roles() {
        let mut user = User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            passkey: "0123456789abcdef0123456789abcdef".to_string(),
            uploaded: 1000,
            downloaded: 500,
            role: "admin".to_string(),
            status: "active".to_string(),
            email_verified: true,
            two_factor_enabled: false,
            two_factor_secret: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };

        assert!(user.is_admin());
        assert!(user.is_moderator());
        assert!(user.is_active());

        user.role = "moderator".to_string();
        assert!(!user.is_admin());
        assert!(user.is_moderator());
    }

    #[test]
    fn test_peer_state() {
        let peer = Peer {
            id: Uuid::new_v4(),
            torrent_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            peer_id: "peer".to_string(),
            ip_address: "127.0.0.1".to_string(),
            port: 6881,
            uploaded: 1000,
            downloaded: 500,
            left: 0,
            state: "seeder".to_string(),
            user_agent: None,
            created_at: Utc::now(),
            last_announce_at: Utc::now(),
        };

        assert!(peer.is_seeder());
        assert!(!peer.is_leecher());
        assert!(peer.is_active());
    }
}
