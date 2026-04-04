use facecam_common::types::DaemonStatus;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, watch, Mutex};
use tracing::debug;

/// Watchdog task — monitors pipeline health and updates uptime
pub async fn run(
    status_tx: Arc<Mutex<watch::Sender<DaemonStatus>>>,
    mut shutdown_rx: broadcast::Receiver<()>,
    start_time: Instant,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Update uptime
                if let Ok(guard) = status_tx.try_lock() {
                    guard.send_modify(|s| {
                        s.uptime_secs = start_time.elapsed().as_secs();
                    });
                }

                // Health checks could go here:
                // - Check if source device still exists in /dev
                // - Check if frames are still flowing (no stuck pipeline)
                // - Check disk space for logs
            }
            _ = shutdown_rx.recv() => {
                debug!("Watchdog shutting down");
                return;
            }
        }
    }
}
