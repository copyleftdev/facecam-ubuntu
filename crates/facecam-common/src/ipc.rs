use crate::types::DaemonStatus;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default socket path for daemon IPC
pub fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("facecam-daemon.sock")
}

/// Commands sent from CLI to daemon via unix socket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonCommand {
    /// Get current status
    Status,
    /// Apply a named profile
    ApplyProfile { name: String },
    /// Set a single control value
    SetControl { name: String, value: i64 },
    /// Get a single control value
    GetControl { name: String },
    /// Get all control values
    GetAllControls,
    /// Export diagnostics bundle
    ExportDiagnostics,
    /// Force USB reset and recovery
    ForceReset,
    /// Restart the pipeline
    RestartPipeline,
    /// Graceful shutdown
    Shutdown,
}

/// Responses sent from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    /// Status snapshot
    Status(DaemonStatus),
    /// Operation succeeded with optional message
    Ok(Option<String>),
    /// Control value
    ControlValue { name: String, value: i64 },
    /// All control values
    Controls(Vec<crate::types::ControlValue>),
    /// Diagnostics bundle path
    DiagnosticsExported(String),
    /// Error
    Error(String),
}
