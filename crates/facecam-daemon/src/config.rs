use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Maximum USB reset recovery attempts before giving up
    #[serde(default = "default_max_recovery")]
    pub max_recovery_attempts: u32,

    /// Timeout in ms waiting for a frame before considering the stream dead
    #[serde(default = "default_frame_timeout")]
    pub frame_timeout_ms: u64,

    /// v4l2loopback module parameters
    #[serde(default)]
    pub loopback: LoopbackConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopbackConfig {
    /// Video device number for the virtual camera
    #[serde(default = "default_video_nr")]
    pub video_nr: u32,

    /// Card label shown to applications
    #[serde(default = "default_card_label")]
    pub card_label: String,

    /// Maximum simultaneous consumers
    #[serde(default = "default_max_openers")]
    pub max_openers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Directory for log files
    #[serde(default = "default_log_dir")]
    pub log_dir: String,

    /// Maximum number of diagnostic events to keep in memory
    #[serde(default = "default_max_events")]
    pub max_events: usize,
}

fn default_max_recovery() -> u32 {
    5
}
fn default_frame_timeout() -> u64 {
    5000
}
fn default_video_nr() -> u32 {
    10
}
fn default_card_label() -> String {
    "Facecam Normalized".to_string()
}
fn default_max_openers() -> u32 {
    10
}
fn default_log_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.local/share/facecam/logs", home)
}
fn default_max_events() -> usize {
    1000
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            max_recovery_attempts: default_max_recovery(),
            frame_timeout_ms: default_frame_timeout(),
            loopback: LoopbackConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            video_nr: default_video_nr(),
            card_label: default_card_label(),
            max_openers: default_max_openers(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: default_log_dir(),
            max_events: default_max_events(),
        }
    }
}

/// Load daemon configuration from file, falling back to defaults
pub fn load_config(path: Option<&str>) -> Result<DaemonConfig> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config/facecam/daemon.toml")
        }
    };

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
        let config: DaemonConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;
        tracing::info!(path = %config_path.display(), "Loaded daemon configuration");
        Ok(config)
    } else {
        tracing::info!("No config file found, using defaults");
        Ok(DaemonConfig::default())
    }
}
