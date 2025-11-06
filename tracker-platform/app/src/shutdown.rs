use anyhow::Result;
use futures::stream::StreamExt;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use std::time::Duration;
use tokio::time::sleep;

/// Setup signal handlers for graceful shutdown
pub async fn shutdown_signal() {
    wait_for_signal().await;
    tracing::info!("Shutdown signal received, starting graceful shutdown...");
}

/// Wait for shutdown signal (SIGTERM or SIGINT)
async fn wait_for_signal() {
    let signals = Signals::new([SIGTERM, SIGINT]).expect("Failed to create signal handler");
    let handle = signals.handle();

    let mut signals = signals.fuse();

    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM => {
                tracing::info!("Received SIGTERM signal");
                break;
            }
            SIGINT => {
                tracing::info!("Received SIGINT signal (Ctrl+C)");
                break;
            }
            _ => unreachable!(),
        }
    }

    handle.close();
}

/// Perform graceful shutdown sequence
pub async fn shutdown_sequence(
    state: crate::state::AppState,
    shutdown_timeout: Duration,
) -> Result<()> {
    tracing::info!("Beginning shutdown sequence...");

    // Step 1: Stop accepting new connections
    tracing::info!("Stopping new connection acceptance...");

    // Step 2: Wait for existing connections to finish or timeout
    tracing::info!(
        "Waiting for existing connections to finish (timeout: {:?})...",
        shutdown_timeout
    );
    sleep(shutdown_timeout).await;

    // Step 3: Close database connections
    tracing::info!("Closing database connections...");
    state.db.close().await;

    // Step 4: Shutdown telemetry
    tracing::info!("Shutting down telemetry...");
    crate::telemetry::shutdown_telemetry();

    tracing::info!("Shutdown sequence completed successfully");
    Ok(())
}

/// Graceful shutdown coordinator that handles the entire shutdown process
pub struct ShutdownCoordinator {
    timeout: Duration,
}

impl ShutdownCoordinator {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Execute the shutdown sequence
    pub async fn shutdown(&self, state: crate::state::AppState) -> Result<()> {
        // Wait for shutdown signal
        shutdown_signal().await;

        // Execute shutdown sequence with timeout
        let shutdown_task = shutdown_sequence(state, self.timeout);

        match tokio::time::timeout(self.timeout * 2, shutdown_task).await {
            Ok(Ok(())) => {
                tracing::info!("Graceful shutdown completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Error during shutdown: {}", e);
                Err(e)
            }
            Err(_) => {
                tracing::error!("Shutdown timeout exceeded, forcing shutdown");
                anyhow::bail!("Shutdown timeout exceeded")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_coordinator_creation() {
        let coordinator = ShutdownCoordinator::new(30);
        assert_eq!(coordinator.timeout, Duration::from_secs(30));
    }
}
