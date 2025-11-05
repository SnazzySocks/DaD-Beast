//! Tracker Statistics and Metrics
//!
//! This module collects real-time statistics about tracker performance and
//! exports them in Prometheus format for monitoring and alerting.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prometheus::{
    Counter, CounterVec, Encoder, Histogram, HistogramOpts, HistogramVec, IntGauge, IntGaugeVec,
    Opts, Registry, TextEncoder,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

/// Tracker statistics collector
///
/// Uses Prometheus metrics for efficient concurrent updates and querying.
/// All metrics use atomic operations internally for thread-safety.
pub struct TrackerStatistics {
    /// Prometheus metrics registry
    registry: Registry,

    // Request counters
    /// Total number of announce requests
    announce_requests: Counter,

    /// Total number of scrape requests
    scrape_requests: Counter,

    /// Number of failed requests
    failed_requests: CounterVec,

    // Response time histograms
    /// Announce request latency histogram
    announce_latency: Histogram,

    /// Scrape request latency histogram
    scrape_latency: Histogram,

    // Peer metrics
    /// Current number of peers across all torrents
    total_peers: IntGauge,

    /// Current number of seeders
    total_seeders: IntGauge,

    /// Current number of leechers
    total_leechers: IntGauge,

    /// Number of active torrents
    active_torrents: IntGauge,

    // Database metrics
    /// Number of batched writes
    batch_writes: Counter,

    /// Number of records written in batches
    batch_records_written: Counter,

    /// Batch write latency histogram
    batch_write_latency: Histogram,

    // Connection metrics
    /// Active HTTP connections
    active_connections: IntGauge,
}

impl TrackerStatistics {
    /// Creates a new statistics collector
    pub fn new() -> Self {
        let registry = Registry::new();

        // Request counters
        let announce_requests = Counter::with_opts(
            Opts::new("tracker_announce_requests_total", "Total number of announce requests")
        ).unwrap();
        registry.register(Box::new(announce_requests.clone())).unwrap();

        let scrape_requests = Counter::with_opts(
            Opts::new("tracker_scrape_requests_total", "Total number of scrape requests")
        ).unwrap();
        registry.register(Box::new(scrape_requests.clone())).unwrap();

        let failed_requests = CounterVec::new(
            Opts::new("tracker_failed_requests_total", "Total number of failed requests"),
            &["type", "reason"]
        ).unwrap();
        registry.register(Box::new(failed_requests.clone())).unwrap();

        // Response time histograms
        let announce_latency = Histogram::with_opts(
            HistogramOpts::new("tracker_announce_duration_seconds", "Announce request duration")
                .buckets(vec![0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0])
        ).unwrap();
        registry.register(Box::new(announce_latency.clone())).unwrap();

        let scrape_latency = Histogram::with_opts(
            HistogramOpts::new("tracker_scrape_duration_seconds", "Scrape request duration")
                .buckets(vec![0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0])
        ).unwrap();
        registry.register(Box::new(scrape_latency.clone())).unwrap();

        // Peer metrics
        let total_peers = IntGauge::with_opts(
            Opts::new("tracker_peers_total", "Total number of peers")
        ).unwrap();
        registry.register(Box::new(total_peers.clone())).unwrap();

        let total_seeders = IntGauge::with_opts(
            Opts::new("tracker_seeders_total", "Total number of seeders")
        ).unwrap();
        registry.register(Box::new(total_seeders.clone())).unwrap();

        let total_leechers = IntGauge::with_opts(
            Opts::new("tracker_leechers_total", "Total number of leechers")
        ).unwrap();
        registry.register(Box::new(total_leechers.clone())).unwrap();

        let active_torrents = IntGauge::with_opts(
            Opts::new("tracker_torrents_active", "Number of active torrents")
        ).unwrap();
        registry.register(Box::new(active_torrents.clone())).unwrap();

        // Database metrics
        let batch_writes = Counter::with_opts(
            Opts::new("tracker_batch_writes_total", "Number of batched database writes")
        ).unwrap();
        registry.register(Box::new(batch_writes.clone())).unwrap();

        let batch_records_written = Counter::with_opts(
            Opts::new("tracker_batch_records_total", "Number of records written in batches")
        ).unwrap();
        registry.register(Box::new(batch_records_written.clone())).unwrap();

        let batch_write_latency = Histogram::with_opts(
            HistogramOpts::new("tracker_batch_write_duration_seconds", "Batch write duration")
                .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        ).unwrap();
        registry.register(Box::new(batch_write_latency.clone())).unwrap();

        // Connection metrics
        let active_connections = IntGauge::with_opts(
            Opts::new("tracker_connections_active", "Number of active HTTP connections")
        ).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();

        Self {
            registry,
            announce_requests,
            scrape_requests,
            failed_requests,
            announce_latency,
            scrape_latency,
            total_peers,
            total_seeders,
            total_leechers,
            active_torrents,
            batch_writes,
            batch_records_written,
            batch_write_latency,
            active_connections,
        }
    }

    /// Records an announce request
    #[inline]
    pub fn record_announce(&self) {
        self.announce_requests.inc();
    }

    /// Records announce request latency
    #[inline]
    pub fn record_announce_latency(&self, duration: Duration) {
        self.announce_latency.observe(duration.as_secs_f64());
    }

    /// Records a scrape request
    #[inline]
    pub fn record_scrape(&self) {
        self.scrape_requests.inc();
    }

    /// Records scrape request latency
    #[inline]
    pub fn record_scrape_latency(&self, duration: Duration) {
        self.scrape_latency.observe(duration.as_secs_f64());
    }

    /// Records a failed request
    #[inline]
    pub fn record_failure(&self, request_type: &str, reason: &str) {
        self.failed_requests
            .with_label_values(&[request_type, reason])
            .inc();
    }

    /// Updates peer counts
    #[inline]
    pub fn update_peer_counts(&self, total: i64, seeders: i64, leechers: i64) {
        self.total_peers.set(total);
        self.total_seeders.set(seeders);
        self.total_leechers.set(leechers);
    }

    /// Updates active torrent count
    #[inline]
    pub fn update_torrent_count(&self, count: i64) {
        self.active_torrents.set(count);
    }

    /// Records a batch write operation
    #[inline]
    pub fn record_batch_write(&self, records: usize, duration: Duration) {
        self.batch_writes.inc();
        self.batch_records_written.inc_by(records as f64);
        self.batch_write_latency.observe(duration.as_secs_f64());
    }

    /// Increments active connection count
    #[inline]
    pub fn connection_opened(&self) {
        self.active_connections.inc();
    }

    /// Decrements active connection count
    #[inline]
    pub fn connection_closed(&self) {
        self.active_connections.dec();
    }

    /// Exports metrics in Prometheus text format
    pub fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Returns the current announce request count
    pub fn announce_count(&self) -> f64 {
        self.announce_requests.get()
    }

    /// Returns the current scrape request count
    pub fn scrape_count(&self) -> f64 {
        self.scrape_requests.get()
    }

    /// Returns announce latency statistics
    pub fn announce_stats(&self) -> (u64, f64) {
        (self.announce_latency.get_sample_count(), self.announce_latency.get_sample_sum())
    }

    /// Returns scrape latency statistics
    pub fn scrape_stats(&self) -> (u64, f64) {
        (self.scrape_latency.get_sample_count(), self.scrape_latency.get_sample_sum())
    }
}

impl Default for TrackerStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP handler for the /metrics endpoint
///
/// Returns Prometheus-formatted metrics for scraping by monitoring systems
pub async fn handle_metrics(
    State(service): State<Arc<crate::TrackerService>>,
) -> Response {
    let stats = service.statistics();

    match stats.export_metrics() {
        Ok(metrics) => (
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            metrics,
        ).into_response(),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export metrics",
            ).into_response()
        }
    }
}

/// Helper struct for tracking request duration
///
/// Automatically records the duration when dropped, ensuring we always
/// capture latency even if the request handler panics or returns early.
pub struct RequestTimer<'a> {
    statistics: &'a TrackerStatistics,
    start: std::time::Instant,
    request_type: RequestType,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    Announce,
    Scrape,
}

impl<'a> RequestTimer<'a> {
    /// Creates a new request timer
    #[inline]
    pub fn new(statistics: &'a TrackerStatistics, request_type: RequestType) -> Self {
        // Record the request
        match request_type {
            RequestType::Announce => statistics.record_announce(),
            RequestType::Scrape => statistics.record_scrape(),
        }

        Self {
            statistics,
            start: std::time::Instant::now(),
            request_type,
        }
    }
}

impl Drop for RequestTimer<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        match self.request_type {
            RequestType::Announce => self.statistics.record_announce_latency(duration),
            RequestType::Scrape => self.statistics.record_scrape_latency(duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_creation() {
        let stats = TrackerStatistics::new();
        assert_eq!(stats.announce_count(), 0.0);
        assert_eq!(stats.scrape_count(), 0.0);
    }

    #[test]
    fn test_record_announce() {
        let stats = TrackerStatistics::new();
        stats.record_announce();
        assert_eq!(stats.announce_count(), 1.0);
    }

    #[test]
    fn test_record_scrape() {
        let stats = TrackerStatistics::new();
        stats.record_scrape();
        assert_eq!(stats.scrape_count(), 1.0);
    }

    #[test]
    fn test_update_peer_counts() {
        let stats = TrackerStatistics::new();
        stats.update_peer_counts(100, 50, 50);
        assert_eq!(stats.total_peers.get(), 100);
        assert_eq!(stats.total_seeders.get(), 50);
        assert_eq!(stats.total_leechers.get(), 50);
    }

    #[test]
    fn test_metrics_export() {
        let stats = TrackerStatistics::new();
        stats.record_announce();
        stats.record_scrape();

        let metrics = stats.export_metrics().unwrap();
        assert!(metrics.contains("tracker_announce_requests_total"));
        assert!(metrics.contains("tracker_scrape_requests_total"));
    }

    #[test]
    fn test_request_timer() {
        let stats = TrackerStatistics::new();
        {
            let _timer = RequestTimer::new(&stats, RequestType::Announce);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(stats.announce_count(), 1.0);

        let (count, sum) = stats.announce_stats();
        assert_eq!(count, 1);
        assert!(sum > 0.0);
    }
}
