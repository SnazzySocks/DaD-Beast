use crate::config::TelemetryConfig;
use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Config, Tracer};
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Initialize telemetry subsystem with tracing and optionally OpenTelemetry
pub fn init_telemetry(config: &TelemetryConfig) -> Result<()> {
    // Set up OpenTelemetry propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create the base subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    // Add formatting layer based on configuration
    let subscriber = match config.log_format.as_str() {
        "json" => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(true)
                .with_span_list(true)
                .with_file(true)
                .with_line_number(true);

            subscriber.with(fmt_layer).boxed()
        }
        _ => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

            subscriber.with(fmt_layer).boxed()
        }
    };

    // Add OpenTelemetry layer if endpoint is configured
    if let Some(otlp_endpoint) = &config.otlp_endpoint {
        tracing::info!("Initializing OpenTelemetry with endpoint: {}", otlp_endpoint);

        let tracer = init_tracer(config, otlp_endpoint)?;
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        subscriber
            .with(telemetry_layer)
            .try_init()
            .context("Failed to initialize tracing subscriber")?;
    } else {
        subscriber
            .try_init()
            .context("Failed to initialize tracing subscriber")?;
    }

    tracing::info!(
        "Telemetry initialized - service: {}, environment: {}, log_level: {}",
        config.service_name,
        config.environment,
        config.log_level
    );

    Ok(())
}

/// Initialize OpenTelemetry tracer
fn init_tracer(config: &TelemetryConfig, endpoint: &str) -> Result<Tracer> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    let trace_config = Config::default().with_resource(Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.environment", config.environment.clone()),
    ]));

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .context("Failed to install OpenTelemetry tracer")?;

    Ok(tracer)
}

/// Shutdown telemetry subsystem gracefully
pub fn shutdown_telemetry() {
    tracing::info!("Shutting down telemetry...");

    // Flush any remaining traces
    global::shutdown_tracer_provider();

    tracing::info!("Telemetry shutdown complete");
}

/// Prometheus metrics registry and exporter
pub mod metrics {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use prometheus::{Encoder, Registry, TextEncoder};
    use std::sync::Arc;

    lazy_static::lazy_static! {
        pub static ref REGISTRY: Registry = Registry::new();

        // HTTP metrics
        pub static ref HTTP_REQUESTS_TOTAL: prometheus::IntCounterVec =
            prometheus::IntCounterVec::new(
                prometheus::opts!("http_requests_total", "Total number of HTTP requests"),
                &["method", "path", "status"]
            ).unwrap();

        pub static ref HTTP_REQUEST_DURATION_SECONDS: prometheus::HistogramVec =
            prometheus::HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "http_request_duration_seconds",
                    "HTTP request duration in seconds"
                ),
                &["method", "path", "status"]
            ).unwrap();

        // Database metrics
        pub static ref DB_QUERIES_TOTAL: prometheus::IntCounterVec =
            prometheus::IntCounterVec::new(
                prometheus::opts!("db_queries_total", "Total number of database queries"),
                &["query_type", "status"]
            ).unwrap();

        pub static ref DB_QUERY_DURATION_SECONDS: prometheus::HistogramVec =
            prometheus::HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "db_query_duration_seconds",
                    "Database query duration in seconds"
                ),
                &["query_type"]
            ).unwrap();

        // Cache metrics
        pub static ref CACHE_REQUESTS_TOTAL: prometheus::IntCounterVec =
            prometheus::IntCounterVec::new(
                prometheus::opts!("cache_requests_total", "Total number of cache requests"),
                &["operation", "status"]
            ).unwrap();

        // Tracker metrics
        pub static ref TRACKER_ANNOUNCES_TOTAL: prometheus::IntCounter =
            prometheus::IntCounter::new(
                "tracker_announces_total",
                "Total number of tracker announces"
            ).unwrap();

        pub static ref TRACKER_SCRAPES_TOTAL: prometheus::IntCounter =
            prometheus::IntCounter::new(
                "tracker_scrapes_total",
                "Total number of tracker scrapes"
            ).unwrap();

        pub static ref ACTIVE_PEERS: prometheus::IntGauge =
            prometheus::IntGauge::new(
                "active_peers",
                "Number of active peers"
            ).unwrap();

        pub static ref ACTIVE_TORRENTS: prometheus::IntGauge =
            prometheus::IntGauge::new(
                "active_torrents",
                "Number of active torrents"
            ).unwrap();
    }

    /// Initialize metrics registry
    pub fn init_metrics() -> anyhow::Result<()> {
        REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
        REGISTRY.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()))?;
        REGISTRY.register(Box::new(DB_QUERIES_TOTAL.clone()))?;
        REGISTRY.register(Box::new(DB_QUERY_DURATION_SECONDS.clone()))?;
        REGISTRY.register(Box::new(CACHE_REQUESTS_TOTAL.clone()))?;
        REGISTRY.register(Box::new(TRACKER_ANNOUNCES_TOTAL.clone()))?;
        REGISTRY.register(Box::new(TRACKER_SCRAPES_TOTAL.clone()))?;
        REGISTRY.register(Box::new(ACTIVE_PEERS.clone()))?;
        REGISTRY.register(Box::new(ACTIVE_TORRENTS.clone()))?;

        Ok(())
    }

    /// Handler for Prometheus metrics endpoint
    pub async fn metrics_handler() -> Response {
        let encoder = TextEncoder::new();
        let metric_families = REGISTRY.gather();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            tracing::error!("Failed to encode metrics: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode metrics",
            )
                .into_response();
        }

        let output = String::from_utf8(buffer).unwrap_or_else(|e| {
            tracing::error!("Failed to convert metrics to string: {}", e);
            String::new()
        });

        (StatusCode::OK, output).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_init() {
        metrics::init_metrics().expect("Failed to initialize metrics");
    }
}
