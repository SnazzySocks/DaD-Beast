//! BitTorrent Scrape Protocol Handler
//!
//! This module handles scrape requests from BitTorrent clients. Scrape
//! requests allow clients to efficiently query statistics for multiple
//! torrents without announcing.

use crate::protocol::{BencodeResponse, InfoHash};
use crate::statistics::{RequestTimer, RequestType};
use crate::TrackerService;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, warn};

/// Maximum number of torrents allowed in a single scrape request
const MAX_SCRAPE_TORRENTS: usize = 100;

/// Scrape request parameters
///
/// Clients can request statistics for multiple torrents by including multiple
/// info_hash parameters in the query string.
///
/// Example: /scrape?info_hash=...&info_hash=...&info_hash=...
#[derive(Debug, Deserialize)]
pub struct ScrapeRequest {
    /// One or more info hashes (20 bytes each, URL-encoded)
    #[serde(rename = "info_hash")]
    pub info_hashes: Option<Vec<String>>,

    /// Passkey for private tracker authentication (optional)
    pub passkey: Option<String>,
}

/// Scrape response error
#[derive(Debug)]
pub struct ScrapeError {
    pub message: String,
    pub status: StatusCode,
}

impl ScrapeError {
    fn new(message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            message: message.into(),
            status,
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(message, StatusCode::BAD_REQUEST)
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

impl IntoResponse for ScrapeError {
    fn into_response(self) -> Response {
        (
            self.status,
            [("Content-Type", "text/plain")],
            self.to_bencode(),
        ).into_response()
    }
}

/// Statistics for a single torrent
#[derive(Debug, Clone)]
pub struct TorrentStats {
    pub complete: i64,    // Number of seeders
    pub incomplete: i64,  // Number of leechers
    pub downloaded: i64,  // Number of completed downloads
}

/// Scrape request handler
pub struct ScrapeHandler {
    service: Arc<TrackerService>,
}

impl ScrapeHandler {
    pub fn new(service: Arc<TrackerService>) -> Self {
        Self { service }
    }

    /// Processes a scrape request
    pub async fn handle(&self, params: ScrapeRequest) -> Result<Vec<u8>, ScrapeError> {
        // Start request timer (automatically records latency on drop)
        let _timer = RequestTimer::new(
            &self.service.statistics(),
            RequestType::Scrape,
        );

        // Get info hashes from request
        let info_hash_strs = params.info_hashes.unwrap_or_default();

        // Validate number of torrents
        if info_hash_strs.is_empty() {
            return Err(ScrapeError::bad_request("No info_hash provided"));
        }

        if info_hash_strs.len() > MAX_SCRAPE_TORRENTS {
            return Err(ScrapeError::bad_request(format!(
                "Too many info_hashes (max: {})",
                MAX_SCRAPE_TORRENTS
            )));
        }

        // Parse info hashes
        let mut info_hashes = Vec::with_capacity(info_hash_strs.len());
        for hash_str in &info_hash_strs {
            match InfoHash::from_urlencoded(hash_str) {
                Ok(hash) => info_hashes.push(hash),
                Err(e) => {
                    warn!("Invalid info_hash in scrape: {}", e);
                    // Skip invalid hashes rather than failing the whole request
                    continue;
                }
            }
        }

        if info_hashes.is_empty() {
            return Err(ScrapeError::bad_request("No valid info_hash provided"));
        }

        // Collect statistics for each torrent
        let mut stats_map: Vec<(InfoHash, Option<TorrentStats>)> = Vec::with_capacity(info_hashes.len());

        for info_hash in info_hashes {
            let stats = self.get_torrent_stats(&info_hash).await;
            stats_map.push((info_hash, stats));
        }

        // Build response
        let response = self.build_scrape_response(&stats_map);

        debug!("Scrape request processed for {} torrents", stats_map.len());

        Ok(response)
    }

    /// Gets statistics for a specific torrent
    async fn get_torrent_stats(&self, info_hash: &InfoHash) -> Option<TorrentStats> {
        // Try to get stats from peer manager (in-memory)
        if let Some((seeders, leechers, completed)) = self.service.peer_manager().get_stats(info_hash) {
            return Some(TorrentStats {
                complete: seeders as i64,
                incomplete: leechers as i64,
                downloaded: completed as i64,
            });
        }

        // If not in memory, could fall back to database query
        // For now, return None if torrent is not being actively tracked
        None
    }

    /// Builds the scrape response in bencode format
    ///
    /// Format:
    /// ```text
    /// d
    ///   5:filesd
    ///     20:<info_hash>d
    ///       8:completei<seeders>e
    ///       10:incompletei<leechers>e
    ///       10:downloadedi<completed>e
    ///     e
    ///     ...
    ///   e
    /// e
    /// ```
    fn build_scrape_response(&self, stats: &[(InfoHash, Option<TorrentStats>)]) -> Vec<u8> {
        // Pre-allocate buffer (typical response is 100-300 bytes per torrent)
        let capacity = 128 + (stats.len() * 128);
        let mut response = BencodeResponse::with_capacity(capacity);

        response.start_dict();

        // "files" dictionary
        response.write_key("files");
        response.start_dict();

        for (info_hash, torrent_stats) in stats {
            // Info hash as key (raw 20 bytes)
            response.write_bytes(info_hash.as_bytes());

            // Stats dictionary
            response.start_dict();

            if let Some(stats) = torrent_stats {
                response.write_key("complete");
                response.write_int(stats.complete);

                response.write_key("incomplete");
                response.write_int(stats.incomplete);

                response.write_key("downloaded");
                response.write_int(stats.downloaded);
            } else {
                // Torrent not found - return zeros
                response.write_key("complete");
                response.write_int(0);

                response.write_key("incomplete");
                response.write_int(0);

                response.write_key("downloaded");
                response.write_int(0);
            }

            response.end_dict();
        }

        response.end_dict();

        // Optional: Add flags
        // response.write_key("flags");
        // response.start_dict();
        // response.write_key("min_request_interval");
        // response.write_int(1800);
        // response.end_dict();

        response.end_dict();

        response.build()
    }
}

/// HTTP handler for scrape requests
///
/// Extracts parameters from the query string and delegates to ScrapeHandler.
pub async fn handle_scrape(
    State(service): State<Arc<TrackerService>>,
    Query(params): Query<ScrapeRequest>,
) -> Result<Response, ScrapeError> {
    let handler = ScrapeHandler::new(service);

    match handler.handle(params).await {
        Ok(response) => Ok((
            StatusCode::OK,
            [("Content-Type", "text/plain")],
            response,
        ).into_response()),
        Err(e) => {
            warn!("Scrape error: {}", e.message);
            service.statistics().record_failure("scrape", &e.message);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_error_bencode() {
        let error = ScrapeError::bad_request("Test error");
        let bencode = error.to_bencode();
        let expected = b"d14:failure reason10:Test errore";
        assert_eq!(bencode, expected);
    }

    #[test]
    fn test_max_scrape_torrents() {
        assert!(MAX_SCRAPE_TORRENTS > 0);
        assert!(MAX_SCRAPE_TORRENTS <= 1000); // Reasonable upper limit
    }

    #[test]
    fn test_torrent_stats_creation() {
        let stats = TorrentStats {
            complete: 10,
            incomplete: 5,
            downloaded: 100,
        };

        assert_eq!(stats.complete, 10);
        assert_eq!(stats.incomplete, 5);
        assert_eq!(stats.downloaded, 100);
    }
}
