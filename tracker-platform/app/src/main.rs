mod config;
mod middleware;
mod routes;
mod shutdown;
mod state;
mod telemetry;

use anyhow::{Context, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::Config::load().context("Failed to load configuration")?;

    // Initialize telemetry (logging, tracing, metrics)
    telemetry::init_telemetry(&config.telemetry)
        .context("Failed to initialize telemetry")?;

    tracing::info!("Starting Tracker Platform...");
    tracing::info!("Environment: {}", config.telemetry.environment);
    tracing::info!("Log level: {}", config.telemetry.log_level);

    // Initialize metrics
    telemetry::metrics::init_metrics().context("Failed to initialize metrics")?;

    // Initialize application state (database, redis, services)
    tracing::info!("Initializing application state...");
    let app_state = state::AppState::new(config.clone())
        .await
        .context("Failed to initialize application state")?;

    tracing::info!("Application state initialized successfully");

    // Build router with all routes and middleware
    let app = routes::build_router(app_state.clone());

    // Apply CORS middleware
    let app = app.layer(middleware::create_cors_layer(&config.cors));

    // Apply compression middleware
    let app = app.layer(middleware::create_compression_layer());

    // Apply tracing middleware
    let app = app.layer(middleware::create_trace_layer());

    // Create socket address
    let addr = SocketAddr::from((
        config
            .server
            .host
            .parse::<std::net::IpAddr>()
            .context("Invalid server host")?,
        config.server.port,
    ));

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("API: http://{}/api/v1", addr);
    tracing::info!("GraphQL: http://{}/graphql", addr);
    tracing::info!("GraphQL Playground: http://{}/graphql/playground", addr);

    if config.telemetry.metrics_enabled {
        if let Some(metrics_port) = config.telemetry.metrics_port {
            tracing::info!("Metrics: http://{}:{}/metrics", config.server.host, metrics_port);
        }
    }

    // Create TCP listener
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    // Set up graceful shutdown
    let shutdown_coordinator =
        shutdown::ShutdownCoordinator::new(config.server.graceful_shutdown_timeout_secs);

    // Spawn the server
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .context("Server error")
    });

    tracing::info!("Tracker Platform started successfully");

    // Wait for shutdown signal
    tokio::select! {
        result = server_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("Server stopped gracefully"),
                Ok(Err(e)) => tracing::error!("Server error: {}", e),
                Err(e) => tracing::error!("Server task panicked: {}", e),
            }
        }
        _ = shutdown_coordinator.shutdown(app_state) => {
            tracing::info!("Shutdown completed");
        }
    }

    tracing::info!("Tracker Platform stopped");

    Ok(())
}
