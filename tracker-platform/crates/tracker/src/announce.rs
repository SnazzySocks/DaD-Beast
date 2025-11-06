//! BitTorrent Announce Protocol Handler
//!
//! This module handles announce requests from BitTorrent clients. Announce
//! requests are the primary way peers discover other peers in the swarm.
//!
//! Target latency: <10ms for optimal client experience

use crate::batch::{PeerUpdate, TorrentUpdate};
use crate::peer::Peer;
use crate::protocol::{BencodeResponse, CompactPeerV4, Event, InfoHash, PeerId};
use crate::statistics::{RequestTimer, RequestType};
use crate::TrackerService;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Default announce interval (seconds)
const DEFAULT_INTERVAL: i64 = 1800; // 30 minutes

/// Minimum announce interval (seconds)
const MIN_INTERVAL: i64 = 900; // 15 minutes

/// Default number of peers to return
const DEFAULT_NUMWANT: usize = 50;

/// Announce request parameters
///
/// These are extracted from the query string of the announce URL.
/// Example: /announce?info_hash=...&peer_id=...&port=6881&uploaded=0&downloaded=0&left=1000
#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    /// Info hash of the torrent (20 bytes, URL-encoded)
    pub info_hash: String,

    /// Peer ID (20 bytes, URL-encoded)
    pub peer_id: String,

    /// Port the peer is listening on
    pub port: u16,

    /// Total bytes uploaded by this peer
    #[serde(default)]
    pub uploaded: u64,

    /// Total bytes downloaded by this peer
    #[serde(default)]
    pub downloaded: u64,

    /// Number of bytes left to download (0 for seeders)
    #[serde(default)]
    pub left: u64,

    /// Event type (started, stopped, completed, or empty)
    #[serde(default)]
    pub event: Option<String>,

    /// Number of peers the client wants to receive
    #[serde(default)]
    pub numwant: Option<i32>,

    /// Whether the client wants compact peer format (always use compact)
    #[serde(default)]
    pub compact: Option<i32>,

    /// Passkey for private tracker authentication
    pub passkey: Option<String>,

    /// Whether client supports IPv6
    #[serde(default)]
    pub ipv6: Option<String>,

    /// Client's reported IP (optional, usually ignored)
    pub ip: Option<String>,

    /// User agent string
    #[serde(rename = "user_agent")]
    pub user_agent: Option<String>,
}

/// Announce response error
#[derive(Debug)]
pub struct AnnounceError {
    pub message: String,
    pub status: StatusCode,
}

impl AnnounceError {
    fn new(message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            message: message.into(),
            status,
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(message, StatusCode::BAD_REQUEST)
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(message, StatusCode::UNAUTHORIZED)
    }

    fn to_bencode(&self) -> Vec<u8> {
        let mut response = BencodeResponse::with_capacity(128);
        response.start_dict();
        response.write_key("failure reason");
        response.write_string(&self.message);
        response.end_dict();
        response.build()
    }
}

impl IntoResponse for AnnounceError {
    fn into_response(self) -> Response {
        (
            self.status,
            [("Content-Type", "text/plain")],
            self.to_bencode(),
        ).into_response()
    }
}

/// Announce request handler
pub struct AnnounceHandler {
    service: Arc<TrackerService>,
}

impl AnnounceHandler {
    pub fn new(service: Arc<TrackerService>) -> Self {
        Self { service }
    }

    /// Processes an announce request
    pub async fn handle(
        &self,
        params: AnnounceRequest,
        client_ip: IpAddr,
    ) -> Result<Vec<u8>, AnnounceError> {
        // Start request timer (automatically records latency on drop)
        let _timer = RequestTimer::new(
            &self.service.statistics(),
            RequestType::Announce,
        );

        // Parse and validate info hash
        let info_hash = InfoHash::from_urlencoded(&params.info_hash)
            .map_err(|e| AnnounceError::bad_request(format!("Invalid info_hash: {}", e)))?;

        // Parse and validate peer ID
        let peer_id = PeerId::from_urlencoded(&params.peer_id)
            .map_err(|e| AnnounceError::bad_request(format!("Invalid peer_id: {}", e)))?;

        // Validate port
        if params.port == 0 {
            return Err(AnnounceError::bad_request("Invalid port"));
        }

        // Authenticate for private tracker (if passkey is required)
        let user_id = if let Some(passkey) = &params.passkey {
            Some(self.authenticate_passkey(passkey).await?)
        } else {
            None
        };

        // Parse event
        let event = params.event
            .as_deref()
            .map(Event::from_str)
            .unwrap_or(Event::None);

        // Determine peer IP (use provided IP or client IP)
        let peer_ip = if let Some(ip_str) = &params.ip {
            ip_str.parse().unwrap_or(client_ip)
        } else {
            client_ip
        };

        // Check if peer is IPv6
        let is_ipv6 = peer_ip.is_ipv6();

        // Get or create swarm for this torrent
        let swarm = self.service.peer_manager().get_or_create_swarm(info_hash);

        // Handle the event
        match event {
            Event::Stopped => {
                // Remove peer from swarm
                swarm.remove_peer(&peer_ip, params.port);
                debug!("Peer stopped: {} for {}", peer_id, info_hash);

                // Return minimal response
                return Ok(self.build_stopped_response());
            }
            Event::Completed => {
                // Increment completed count
                swarm.increment_completed();
                debug!("Peer completed: {} for {}", peer_id, info_hash);
            }
            _ => {}
        }

        // Create or update peer
        let peer = Peer::new(
            peer_id,
            user_id,
            peer_ip,
            params.port,
            params.uploaded,
            params.downloaded,
            params.left,
        );

        let is_seeder = peer.is_seeder;

        // Update swarm
        swarm.upsert_peer(peer.clone());

        // Queue database update (batched write)
        self.service.batch_writer().queue_peer_update(PeerUpdate {
            info_hash,
            peer_id,
            user_id,
            ip: peer_ip,
            port: params.port,
            uploaded: params.uploaded,
            downloaded: params.downloaded,
            left: params.left,
            is_seeder,
            user_agent: params.user_agent.clone(),
        });

        // Queue torrent stats update
        self.service.batch_writer().queue_torrent_update(TorrentUpdate {
            info_hash,
            seeders: swarm.seeder_count() as i32,
            leechers: swarm.leecher_count() as i32,
            completed_delta: if event == Event::Completed { 1 } else { 0 },
        });

        // Select peers to return
        let numwant = params.numwant
            .map(|n| n.max(0) as usize)
            .unwrap_or(DEFAULT_NUMWANT);

        let peers = swarm.select_peers(is_seeder, is_ipv6, numwant);

        // Build response
        let response = self.build_announce_response(
            &peers,
            swarm.seeder_count() as i64,
            swarm.leecher_count() as i64,
            is_ipv6,
        );

        Ok(response)
    }

    /// Authenticates a passkey and returns the user ID
    async fn authenticate_passkey(&self, passkey: &str) -> Result<Uuid, AnnounceError> {
        // TODO: Implement actual passkey validation against database
        // For now, try to parse as UUID
        passkey.parse()
            .map_err(|_| AnnounceError::unauthorized("Invalid passkey"))
    }

    /// Builds the announce response
    fn build_announce_response(
        &self,
        peers: &[Peer],
        seeders: i64,
        leechers: i64,
        ipv6: bool,
    ) -> Vec<u8> {
        // Pre-allocate buffer (typical response is 300-500 bytes)
        let mut response = BencodeResponse::with_capacity(512);

        response.start_dict();

        // Interval
        response.write_key("interval");
        response.write_int(DEFAULT_INTERVAL);

        // Min interval
        response.write_key("min interval");
        response.write_int(MIN_INTERVAL);

        // Tracker ID (optional, for stateless trackers)
        // response.write_key("tracker id");
        // response.write_string("tracker-001");

        // Complete (seeders)
        response.write_key("complete");
        response.write_int(seeders);

        // Incomplete (leechers)
        response.write_key("incomplete");
        response.write_int(leechers);

        // Peers in compact format
        response.write_key("peers");

        if ipv6 {
            // IPv6 compact format (18 bytes per peer)
            let peer_bytes: Vec<u8> = peers
                .iter()
                .filter_map(|p| p.to_compact_v6())
                .flat_map(|p| p.encode())
                .collect();
            response.write_bytes(&peer_bytes);
        } else {
            // IPv4 compact format (6 bytes per peer)
            let peer_bytes: Vec<u8> = peers
                .iter()
                .filter_map(|p| p.to_compact_v4())
                .flat_map(|p| p.encode())
                .collect();
            response.write_bytes(&peer_bytes);
        }

        response.end_dict();

        response.build()
    }

    /// Builds a minimal response for stopped events
    fn build_stopped_response(&self) -> Vec<u8> {
        let mut response = BencodeResponse::with_capacity(64);
        response.start_dict();
        response.write_key("interval");
        response.write_int(DEFAULT_INTERVAL);
        response.end_dict();
        response.build()
    }
}

/// HTTP handler for announce requests
///
/// Extracts parameters from the query string and the client's IP address,
/// then delegates to AnnounceHandler.
pub async fn handle_announce(
    State(service): State<Arc<TrackerService>>,
    Query(params): Query<AnnounceRequest>,
) -> Result<Response, AnnounceError> {
    // TODO: Extract real client IP from headers (X-Forwarded-For, X-Real-IP)
    // For now, use a placeholder
    let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

    let handler = AnnounceHandler::new(service);

    match handler.handle(params, client_ip).await {
        Ok(response) => Ok((
            StatusCode::OK,
            [("Content-Type", "text/plain")],
            response,
        ).into_response()),
        Err(e) => {
            warn!("Announce error: {}", e.message);
            service.statistics().record_failure("announce", &e.message);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announce_error_bencode() {
        let error = AnnounceError::bad_request("Test error");
        let bencode = error.to_bencode();
        let expected = b"d14:failure reason10:Test errore";
        assert_eq!(bencode, expected);
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_INTERVAL, 1800);
        assert_eq!(MIN_INTERVAL, 900);
        assert!(DEFAULT_NUMWANT > 0);
    }
}
