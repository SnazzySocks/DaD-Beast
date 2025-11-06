//! Peer Management
//!
//! This module handles peer state, selection algorithms, and expiry. It uses
//! efficient concurrent data structures (DashMap) to allow lock-free reads and
//! writes from multiple threads.

use crate::protocol::{InfoHash, PeerId, CompactPeerV4, CompactPeerV6};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use uuid::Uuid;

/// Default peer timeout - peers not seen for this duration are considered stale
pub const PEER_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

/// Maximum number of peers to return in an announce response
pub const MAX_PEERS_RETURNED: usize = 50;

/// Represents a peer in the swarm
///
/// Contains all state needed to track a peer's participation in a torrent.
/// Optimized for memory efficiency while maintaining necessary metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Unique peer identifier
    pub peer_id: PeerId,

    /// User ID associated with this peer (for private trackers)
    pub user_id: Option<Uuid>,

    /// IP address of the peer
    pub ip: IpAddr,

    /// Port the peer is listening on
    pub port: u16,

    /// Total bytes uploaded by this peer
    pub uploaded: u64,

    /// Total bytes downloaded by this peer
    pub downloaded: u64,

    /// Number of bytes left to download (0 for seeders)
    pub left: u64,

    /// Timestamp of last announce
    pub last_seen: DateTime<Utc>,

    /// Whether the peer is a seeder (left == 0)
    pub is_seeder: bool,

    /// User agent string if provided
    pub user_agent: Option<String>,
}

impl Peer {
    /// Creates a new peer with the given parameters
    #[inline]
    pub fn new(
        peer_id: PeerId,
        user_id: Option<Uuid>,
        ip: IpAddr,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
    ) -> Self {
        Self {
            peer_id,
            user_id,
            ip,
            port,
            uploaded,
            downloaded,
            left,
            last_seen: Utc::now(),
            is_seeder: left == 0,
            user_agent: None,
        }
    }

    /// Updates peer statistics from an announce request
    #[inline]
    pub fn update(&mut self, uploaded: u64, downloaded: u64, left: u64) {
        self.uploaded = uploaded;
        self.downloaded = downloaded;
        self.left = left;
        self.is_seeder = left == 0;
        self.last_seen = Utc::now();
    }

    /// Checks if the peer has timed out
    #[inline]
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_seen);
        elapsed.num_seconds() as u64 > PEER_TIMEOUT.as_secs()
    }

    /// Converts to compact IPv4 format if applicable
    #[inline]
    pub fn to_compact_v4(&self) -> Option<CompactPeerV4> {
        match self.ip {
            IpAddr::V4(ip) => Some(CompactPeerV4::new(ip, self.port)),
            IpAddr::V6(_) => None,
        }
    }

    /// Converts to compact IPv6 format if applicable
    #[inline]
    pub fn to_compact_v6(&self) -> Option<CompactPeerV6> {
        match self.ip {
            IpAddr::V6(ip) => Some(CompactPeerV6::new(ip, self.port)),
            IpAddr::V4(_) => None,
        }
    }
}

/// Swarm represents all peers for a specific torrent
///
/// Uses DashMap for lock-free concurrent access. The key is the peer's IP:port
/// combination as a string for fast lookups.
#[derive(Debug)]
pub struct Swarm {
    /// Map of peer_key -> Peer
    peers: DashMap<String, Peer>,

    /// Cached count of seeders (updated on modifications)
    seeder_count: AtomicU64,

    /// Cached count of leechers (updated on modifications)
    leecher_count: AtomicU64,

    /// Total number of completed downloads
    completed: AtomicU64,
}

impl Swarm {
    /// Creates a new empty swarm
    pub fn new() -> Self {
        Self {
            peers: DashMap::new(),
            seeder_count: AtomicU64::new(0),
            leecher_count: AtomicU64::new(0),
            completed: AtomicU64::new(0),
        }
    }

    /// Generates a peer key from IP and port
    #[inline]
    fn peer_key(ip: &IpAddr, port: u16) -> String {
        format!("{}:{}", ip, port)
    }

    /// Adds or updates a peer in the swarm
    ///
    /// Returns true if this was a new peer, false if it was an update
    pub fn upsert_peer(&self, peer: Peer) -> bool {
        let key = Self::peer_key(&peer.ip, peer.port);
        let is_seeder = peer.is_seeder;

        let is_new = if let Some(mut existing) = self.peers.get_mut(&key) {
            let was_seeder = existing.is_seeder;
            existing.update(peer.uploaded, peer.downloaded, peer.left);

            // Update counts if seeder status changed
            if was_seeder != is_seeder {
                if is_seeder {
                    self.seeder_count.fetch_add(1, Ordering::Relaxed);
                    self.leecher_count.fetch_sub(1, Ordering::Relaxed);
                } else {
                    self.seeder_count.fetch_sub(1, Ordering::Relaxed);
                    self.leecher_count.fetch_add(1, Ordering::Relaxed);
                }
            }

            false
        } else {
            // New peer
            self.peers.insert(key, peer);

            if is_seeder {
                self.seeder_count.fetch_add(1, Ordering::Relaxed);
            } else {
                self.leecher_count.fetch_add(1, Ordering::Relaxed);
            }

            true
        };

        is_new
    }

    /// Removes a peer from the swarm
    pub fn remove_peer(&self, ip: &IpAddr, port: u16) -> Option<Peer> {
        let key = Self::peer_key(ip, port);

        if let Some((_, peer)) = self.peers.remove(&key) {
            if peer.is_seeder {
                self.seeder_count.fetch_sub(1, Ordering::Relaxed);
            } else {
                self.leecher_count.fetch_sub(1, Ordering::Relaxed);
            }
            Some(peer)
        } else {
            None
        }
    }

    /// Marks a download as completed
    #[inline]
    pub fn increment_completed(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the number of seeders
    #[inline]
    pub fn seeder_count(&self) -> u64 {
        self.seeder_count.load(Ordering::Relaxed)
    }

    /// Returns the number of leechers
    #[inline]
    pub fn leecher_count(&self) -> u64 {
        self.leecher_count.load(Ordering::Relaxed)
    }

    /// Returns the total number of completed downloads
    #[inline]
    pub fn completed_count(&self) -> u64 {
        self.completed.load(Ordering::Relaxed)
    }

    /// Returns the total number of peers
    #[inline]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Selects peers to return in an announce response
    ///
    /// Implements a round-robin-like selection inspired by Ocelot:
    /// - Prefers peers of the opposite type (seeders get leechers, vice versa)
    /// - Returns up to MAX_PEERS_RETURNED peers
    /// - Filters by IP version (v4 or v6)
    pub fn select_peers(&self, is_seeder: bool, ipv6: bool, numwant: usize) -> Vec<Peer> {
        let numwant = numwant.min(MAX_PEERS_RETURNED);
        let mut selected = Vec::with_capacity(numwant);

        // First pass: select peers of opposite type
        let prefer_seeders = !is_seeder;
        for entry in self.peers.iter() {
            if selected.len() >= numwant {
                break;
            }

            let peer = entry.value();

            // Filter by IP version
            let ip_matches = match (&peer.ip, ipv6) {
                (IpAddr::V4(_), false) => true,
                (IpAddr::V6(_), true) => true,
                _ => false,
            };

            if !ip_matches {
                continue;
            }

            // Skip expired peers
            if peer.is_expired() {
                continue;
            }

            // Prefer opposite type
            if peer.is_seeder == prefer_seeders {
                selected.push(peer.clone());
            }
        }

        // Second pass: fill with any remaining peers if needed
        if selected.len() < numwant {
            for entry in self.peers.iter() {
                if selected.len() >= numwant {
                    break;
                }

                let peer = entry.value();

                // Filter by IP version
                let ip_matches = match (&peer.ip, ipv6) {
                    (IpAddr::V4(_), false) => true,
                    (IpAddr::V6(_), true) => true,
                    _ => false,
                };

                if !ip_matches || peer.is_expired() {
                    continue;
                }

                // Add if not already selected
                if !selected.iter().any(|p| p.peer_id == peer.peer_id) {
                    selected.push(peer.clone());
                }
            }
        }

        selected
    }

    /// Removes all expired peers
    ///
    /// Should be called periodically to clean up stale peers
    pub fn cleanup_expired(&self) -> usize {
        let mut removed = 0;

        self.peers.retain(|_, peer| {
            if peer.is_expired() {
                // Update counts
                if peer.is_seeder {
                    self.seeder_count.fetch_sub(1, Ordering::Relaxed);
                } else {
                    self.leecher_count.fetch_sub(1, Ordering::Relaxed);
                }
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}

/// Peer manager coordinates all swarms across all torrents
///
/// Uses a DashMap with InfoHash as key for lock-free concurrent access
pub struct PeerManager {
    /// Map of info_hash -> Swarm
    swarms: DashMap<InfoHash, Swarm>,
}

impl PeerManager {
    /// Creates a new peer manager
    pub fn new() -> Self {
        Self {
            swarms: DashMap::new(),
        }
    }

    /// Gets or creates a swarm for the given info hash
    pub fn get_or_create_swarm(&self, info_hash: InfoHash) -> dashmap::mapref::one::Ref<InfoHash, Swarm> {
        self.swarms.entry(info_hash).or_insert_with(Swarm::new);
        self.swarms.get(&info_hash).unwrap()
    }

    /// Adds or updates a peer in the appropriate swarm
    pub fn upsert_peer(&self, info_hash: InfoHash, peer: Peer) -> bool {
        let swarm = self.get_or_create_swarm(info_hash);
        swarm.upsert_peer(peer)
    }

    /// Removes a peer from a swarm
    pub fn remove_peer(&self, info_hash: InfoHash, ip: &IpAddr, port: u16) -> Option<Peer> {
        self.swarms.get(&info_hash)?.remove_peer(ip, port)
    }

    /// Gets statistics for a torrent
    pub fn get_stats(&self, info_hash: &InfoHash) -> Option<(u64, u64, u64)> {
        self.swarms.get(info_hash).map(|swarm| {
            (
                swarm.seeder_count(),
                swarm.leecher_count(),
                swarm.completed_count(),
            )
        })
    }

    /// Cleans up expired peers across all swarms
    ///
    /// Returns the total number of peers removed
    pub fn cleanup_all_expired(&self) -> usize {
        let mut total_removed = 0;

        for entry in self.swarms.iter() {
            total_removed += entry.value().cleanup_expired();
        }

        total_removed
    }

    /// Returns the total number of swarms
    #[inline]
    pub fn swarm_count(&self) -> usize {
        self.swarms.len()
    }

    /// Returns the total number of peers across all swarms
    pub fn total_peer_count(&self) -> usize {
        self.swarms.iter()
            .map(|entry| entry.value().peer_count())
            .sum()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_peer(ip: &str, port: u16, left: u64) -> Peer {
        Peer::new(
            PeerId::new(*b"12345678901234567890"),
            None,
            ip.parse().unwrap(),
            port,
            0,
            0,
            left,
        )
    }

    #[test]
    fn test_peer_is_seeder() {
        let seeder = create_test_peer("192.168.1.1", 6881, 0);
        assert!(seeder.is_seeder);

        let leecher = create_test_peer("192.168.1.2", 6882, 1000);
        assert!(!leecher.is_seeder);
    }

    #[test]
    fn test_swarm_upsert() {
        let swarm = Swarm::new();
        let peer = create_test_peer("192.168.1.1", 6881, 0);

        assert!(swarm.upsert_peer(peer.clone()));
        assert_eq!(swarm.seeder_count(), 1);
        assert_eq!(swarm.leecher_count(), 0);

        assert!(!swarm.upsert_peer(peer));
        assert_eq!(swarm.seeder_count(), 1);
    }

    #[test]
    fn test_swarm_remove_peer() {
        let swarm = Swarm::new();
        let peer = create_test_peer("192.168.1.1", 6881, 1000);

        swarm.upsert_peer(peer.clone());
        assert_eq!(swarm.leecher_count(), 1);

        let removed = swarm.remove_peer(&peer.ip, peer.port);
        assert!(removed.is_some());
        assert_eq!(swarm.leecher_count(), 0);
    }

    #[test]
    fn test_peer_selection() {
        let swarm = Swarm::new();

        // Add some seeders and leechers
        for i in 0..10 {
            let peer = create_test_peer(&format!("192.168.1.{}", i), 6881, 0);
            swarm.upsert_peer(peer);
        }

        for i in 10..20 {
            let peer = create_test_peer(&format!("192.168.1.{}", i), 6881, 1000);
            swarm.upsert_peer(peer);
        }

        // Leecher should get seeders
        let peers = swarm.select_peers(false, false, 5);
        assert!(peers.len() <= 5);
        assert!(peers.iter().all(|p| p.is_seeder));

        // Seeder should get leechers
        let peers = swarm.select_peers(true, false, 5);
        assert!(peers.len() <= 5);
        assert!(peers.iter().all(|p| !p.is_seeder));
    }

    #[test]
    fn test_peer_manager() {
        let manager = PeerManager::new();
        let info_hash = InfoHash::new([1u8; 20]);
        let peer = create_test_peer("192.168.1.1", 6881, 0);

        manager.upsert_peer(info_hash, peer);

        let stats = manager.get_stats(&info_hash);
        assert_eq!(stats, Some((1, 0, 0)));
    }
}
