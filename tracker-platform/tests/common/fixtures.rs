/// Test data fixtures
use chrono::{DateTime, Utc};
use fake::{Fake, Faker};
use fake::faker::internet::en::*;
use fake::faker::name::en::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Test user fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl TestUser {
    pub fn new(username: Option<String>, email: Option<String>) -> Self {
        let username = username.unwrap_or_else(|| Username().fake());
        let email = email.unwrap_or_else(|| SafeEmail().fake());
        let password = "TestPassword123!".to_string();

        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password: password.clone(),
            password_hash: hash_password(&password),
            is_active: true,
            is_verified: true,
            created_at: Utc::now(),
        }
    }

    pub fn unverified() -> Self {
        let mut user = Self::new(None, None);
        user.is_verified = false;
        user
    }

    pub fn inactive() -> Self {
        let mut user = Self::new(None, None);
        user.is_active = false;
        user
    }
}

/// Test torrent fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTorrent {
    pub id: Uuid,
    pub info_hash: String,
    pub name: String,
    pub description: String,
    pub size: i64,
    pub uploader_id: Uuid,
    pub category_id: i32,
    pub is_approved: bool,
    pub seeders: i32,
    pub leechers: i32,
    pub downloads: i32,
    pub created_at: DateTime<Utc>,
}

impl TestTorrent {
    pub fn new(uploader_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            info_hash: generate_info_hash(),
            name: format!("Test Torrent {}", Uuid::new_v4()),
            description: "Test torrent description".to_string(),
            size: (1_000_000..10_000_000_000).fake(),
            uploader_id,
            category_id: (1..10).fake(),
            is_approved: true,
            seeders: (0..100).fake(),
            leechers: (0..50).fake(),
            downloads: (0..1000).fake(),
            created_at: Utc::now(),
        }
    }

    pub fn unapproved(uploader_id: Uuid) -> Self {
        let mut torrent = Self::new(uploader_id);
        torrent.is_approved = false;
        torrent
    }
}

/// Test peer fixture
#[derive(Debug, Clone)]
pub struct TestPeer {
    pub peer_id: String,
    pub ip: String,
    pub port: u16,
    pub uploaded: i64,
    pub downloaded: i64,
    pub left: i64,
    pub event: String,
}

impl TestPeer {
    pub fn new() -> Self {
        Self {
            peer_id: generate_peer_id(),
            ip: format!("{}.{}.{}.{}",
                (1..255).fake::<u8>(),
                (0..255).fake::<u8>(),
                (0..255).fake::<u8>(),
                (1..255).fake::<u8>()
            ),
            port: (1024..65535).fake(),
            uploaded: 0,
            downloaded: 0,
            left: (1_000_000..10_000_000).fake(),
            event: "started".to_string(),
        }
    }

    pub fn seeder() -> Self {
        let mut peer = Self::new();
        peer.left = 0;
        peer.uploaded = (1_000_000..100_000_000).fake();
        peer
    }

    pub fn leecher() -> Self {
        let mut peer = Self::new();
        peer.left = (100_000..10_000_000).fake();
        peer.downloaded = (10_000..1_000_000).fake();
        peer
    }
}

impl Default for TestPeer {
    fn default() -> Self {
        Self::new()
    }
}

/// Test forum post fixture
#[derive(Debug, Clone)]
pub struct TestForumPost {
    pub id: Uuid,
    pub forum_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl TestForumPost {
    pub fn new(forum_id: Uuid, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            forum_id,
            user_id,
            title: format!("Test Post {}", Uuid::new_v4()),
            content: "This is a test forum post content.".to_string(),
            created_at: Utc::now(),
        }
    }
}

/// Generate a random 40-character hex info hash
fn generate_info_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..40)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

/// Generate a random 20-character peer ID
fn generate_peer_id() -> String {
    format!("-TR3000-{}",
        (0..12)
            .map(|_| rand::random::<u8>() % 10)
            .map(|n| n.to_string())
            .collect::<String>()
    )
}

/// Hash password using bcrypt
fn hash_password(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap()
}

/// Builder for batch test data creation
pub struct FixtureBuilder {
    test_id: String,
}

impl FixtureBuilder {
    pub fn new(test_id: String) -> Self {
        Self { test_id }
    }

    /// Create multiple test users
    pub fn users(&self, count: usize) -> Vec<TestUser> {
        (0..count)
            .map(|i| {
                TestUser::new(
                    Some(format!("testuser_{}_{}", self.test_id, i)),
                    Some(format!("test_{}_{}@example.com", self.test_id, i))
                )
            })
            .collect()
    }

    /// Create multiple test torrents
    pub fn torrents(&self, uploader_id: Uuid, count: usize) -> Vec<TestTorrent> {
        (0..count)
            .map(|_| TestTorrent::new(uploader_id))
            .collect()
    }

    /// Create multiple test peers
    pub fn peers(&self, count: usize) -> Vec<TestPeer> {
        (0..count)
            .map(|_| TestPeer::new())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_fixture() {
        let user = TestUser::new(None, None);
        assert!(!user.username.is_empty());
        assert!(user.email.contains('@'));
        assert!(user.is_active);
        assert!(user.is_verified);
    }

    #[test]
    fn test_torrent_fixture() {
        let user_id = Uuid::new_v4();
        let torrent = TestTorrent::new(user_id);
        assert_eq!(torrent.uploader_id, user_id);
        assert_eq!(torrent.info_hash.len(), 40);
    }

    #[test]
    fn test_peer_fixture() {
        let peer = TestPeer::new();
        assert_eq!(peer.peer_id.len(), 20);
        assert!(peer.port > 1024);
    }

    #[test]
    fn test_fixture_builder() {
        let builder = FixtureBuilder::new("test123".to_string());
        let users = builder.users(5);
        assert_eq!(users.len(), 5);

        let torrents = builder.torrents(Uuid::new_v4(), 3);
        assert_eq!(torrents.len(), 3);
    }
}
