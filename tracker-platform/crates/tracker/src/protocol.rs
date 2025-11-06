//! BitTorrent Protocol Utilities
//!
//! This module provides core protocol types and encoding/decoding utilities
//! for the BitTorrent tracker protocol. It implements efficient bencode
//! encoding/decoding and compact peer format handling.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// 20-byte SHA1 info hash identifying a torrent
///
/// The info hash is the SHA1 hash of the 'info' dictionary in a .torrent file.
/// It uniquely identifies a torrent across the BitTorrent network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InfoHash(pub [u8; 20]);

impl InfoHash {
    /// Creates an InfoHash from a 20-byte array
    #[inline]
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Parses an InfoHash from a hex string
    pub fn from_hex(s: &str) -> Result<Self> {
        if s.len() != 40 {
            return Err(anyhow!("Info hash must be 40 hex characters"));
        }

        let mut bytes = [0u8; 20];
        for i in 0..20 {
            bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?;
        }

        Ok(Self(bytes))
    }

    /// Parses an InfoHash from URL-encoded bytes
    ///
    /// Handles both URL-encoded (%XX) and raw binary formats
    pub fn from_urlencoded(s: &str) -> Result<Self> {
        let decoded = urlencoding::decode_binary(s.as_bytes());
        if decoded.len() != 20 {
            return Err(anyhow!("Info hash must be 20 bytes, got {}", decoded.len()));
        }

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded);
        Ok(Self(bytes))
    }

    /// Returns the raw bytes of the info hash
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Converts to a hex string representation
    pub fn to_hex(&self) -> String {
        self.0.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// 20-byte peer ID identifying a BitTorrent client
///
/// Peer IDs typically start with a client identifier (e.g., "-DE13A0-" for Deluge)
/// followed by random bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 20]);

impl PeerId {
    /// Creates a PeerId from a 20-byte array
    #[inline]
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Parses a PeerId from URL-encoded bytes
    pub fn from_urlencoded(s: &str) -> Result<Self> {
        let decoded = urlencoding::decode_binary(s.as_bytes());
        if decoded.len() != 20 {
            return Err(anyhow!("Peer ID must be 20 bytes, got {}", decoded.len()));
        }

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded);
        Ok(Self(bytes))
    }

    /// Returns the raw bytes of the peer ID
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Extracts the client identifier prefix if present
    pub fn client_prefix(&self) -> Option<&str> {
        if self.0[0] == b'-' && self.0[7] == b'-' {
            std::str::from_utf8(&self.0[1..7]).ok()
        } else {
            None
        }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

/// Compact peer format for IPv4 (6 bytes: 4 bytes IP + 2 bytes port)
///
/// This is the standard compact format used in BitTorrent announce responses.
/// It's significantly more efficient than the dictionary format, reducing
/// bandwidth and parsing overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactPeerV4 {
    pub ip: [u8; 4],
    pub port: u16,
}

impl CompactPeerV4 {
    /// Creates a compact peer from IP and port
    #[inline]
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self {
            ip: ip.octets(),
            port,
        }
    }

    /// Encodes the peer into 6 bytes
    ///
    /// Format: [IP0, IP1, IP2, IP3, PORT_HIGH, PORT_LOW]
    #[inline]
    pub fn encode(&self) -> [u8; 6] {
        let mut bytes = [0u8; 6];
        bytes[0..4].copy_from_slice(&self.ip);
        bytes[4..6].copy_from_slice(&self.port.to_be_bytes());
        bytes
    }

    /// Decodes a peer from 6 bytes
    #[inline]
    pub fn decode(bytes: &[u8; 6]) -> Self {
        let ip = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let port = u16::from_be_bytes([bytes[4], bytes[5]]);
        Self { ip, port }
    }

    /// Converts to a SocketAddr
    #[inline]
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from(self.ip)),
            self.port,
        )
    }
}

/// Compact peer format for IPv6 (18 bytes: 16 bytes IP + 2 bytes port)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactPeerV6 {
    pub ip: [u8; 16],
    pub port: u16,
}

impl CompactPeerV6 {
    /// Creates a compact peer from IPv6 address and port
    #[inline]
    pub fn new(ip: Ipv6Addr, port: u16) -> Self {
        Self {
            ip: ip.octets(),
            port,
        }
    }

    /// Encodes the peer into 18 bytes
    #[inline]
    pub fn encode(&self) -> [u8; 18] {
        let mut bytes = [0u8; 18];
        bytes[0..16].copy_from_slice(&self.ip);
        bytes[16..18].copy_from_slice(&self.port.to_be_bytes());
        bytes
    }

    /// Decodes a peer from 18 bytes
    #[inline]
    pub fn decode(bytes: &[u8; 18]) -> Self {
        let mut ip = [0u8; 16];
        ip.copy_from_slice(&bytes[0..16]);
        let port = u16::from_be_bytes([bytes[16], bytes[17]]);
        Self { ip, port }
    }

    /// Converts to a SocketAddr
    #[inline]
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            IpAddr::V6(Ipv6Addr::from(self.ip)),
            self.port,
        )
    }
}

/// Encode a list of IPv4 peers into compact format
///
/// Returns a byte vector where each peer occupies 6 bytes
pub fn encode_compact_peers_v4(peers: &[CompactPeerV4]) -> Vec<u8> {
    let mut result = Vec::with_capacity(peers.len() * 6);
    for peer in peers {
        result.extend_from_slice(&peer.encode());
    }
    result
}

/// Encode a list of IPv6 peers into compact format
///
/// Returns a byte vector where each peer occupies 18 bytes
pub fn encode_compact_peers_v6(peers: &[CompactPeerV6]) -> Vec<u8> {
    let mut result = Vec::with_capacity(peers.len() * 18);
    for peer in peers {
        result.extend_from_slice(&peer.encode());
    }
    result
}

/// BitTorrent event type sent in announce requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    /// First request - peer is starting download
    Started,
    /// Peer is stopping the download
    Stopped,
    /// Peer has completed the download (became a seeder)
    Completed,
    /// Regular periodic update (or omitted from request)
    None,
}

impl Event {
    /// Parses an event from a query parameter string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "started" => Event::Started,
            "stopped" => Event::Stopped,
            "completed" => Event::Completed,
            _ => Event::None,
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Event::None
    }
}

/// Bencode response builder for efficient response generation
///
/// Pre-allocates buffers and provides optimized encoding for announce/scrape
/// responses to minimize allocations in the hot path.
pub struct BencodeResponse {
    buffer: Vec<u8>,
}

impl BencodeResponse {
    /// Creates a new response builder with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Starts a dictionary
    #[inline]
    pub fn start_dict(&mut self) {
        self.buffer.push(b'd');
    }

    /// Ends a dictionary
    #[inline]
    pub fn end_dict(&mut self) {
        self.buffer.push(b'e');
    }

    /// Writes a string key
    #[inline]
    pub fn write_key(&mut self, key: &str) {
        self.write_string(key);
    }

    /// Writes a string value
    #[inline]
    pub fn write_string(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.len().to_string().as_bytes());
        self.buffer.push(b':');
        self.buffer.extend_from_slice(s.as_bytes());
    }

    /// Writes an integer value
    #[inline]
    pub fn write_int(&mut self, n: i64) {
        self.buffer.push(b'i');
        self.buffer.extend_from_slice(n.to_string().as_bytes());
        self.buffer.push(b'e');
    }

    /// Writes raw bytes with length prefix
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes.len().to_string().as_bytes());
        self.buffer.push(b':');
        self.buffer.extend_from_slice(bytes);
    }

    /// Consumes the builder and returns the encoded bytes
    #[inline]
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_hash_from_hex() {
        let hex = "0123456789abcdef0123456789abcdef01234567";
        let hash = InfoHash::from_hex(hex).unwrap();
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_compact_peer_v4_encoding() {
        let peer = CompactPeerV4::new(Ipv4Addr::new(192, 168, 1, 1), 6881);
        let encoded = peer.encode();
        let decoded = CompactPeerV4::decode(&encoded);
        assert_eq!(peer, decoded);
    }

    #[test]
    fn test_compact_peer_v6_encoding() {
        let peer = CompactPeerV6::new(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1),
            6881,
        );
        let encoded = peer.encode();
        let decoded = CompactPeerV6::decode(&encoded);
        assert_eq!(peer, decoded);
    }

    #[test]
    fn test_event_parsing() {
        assert_eq!(Event::from_str("started"), Event::Started);
        assert_eq!(Event::from_str("stopped"), Event::Stopped);
        assert_eq!(Event::from_str("completed"), Event::Completed);
        assert_eq!(Event::from_str(""), Event::None);
        assert_eq!(Event::from_str("invalid"), Event::None);
    }

    #[test]
    fn test_bencode_response() {
        let mut response = BencodeResponse::with_capacity(64);
        response.start_dict();
        response.write_key("interval");
        response.write_int(1800);
        response.write_key("complete");
        response.write_int(5);
        response.end_dict();

        let expected = b"d8:intervali1800e8:completei5ee";
        assert_eq!(response.build(), expected);
    }

    #[test]
    fn test_peer_id_client_prefix() {
        let peer_id = PeerId::new(*b"-DE13A0-xxxxxxxxxxxx");
        assert_eq!(peer_id.client_prefix(), Some("DE13A0"));

        let peer_id = PeerId::new(*b"M4-4-0--xxxxxxxxxxxx");
        assert_eq!(peer_id.client_prefix(), None);
    }
}
