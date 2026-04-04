use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// A named camera control profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Preferred video mode (if unset, daemon picks best available)
    pub video_mode: Option<ProfileVideoMode>,
    /// Control values to apply when this profile is activated
    #[serde(default)]
    pub controls: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVideoMode {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// Preferred pixel format (defaults to YUYV)
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "YUYV".to_string()
}

/// Profile storage directory
pub fn profiles_dir() -> PathBuf {
    let config_dir = dirs_path();
    config_dir.join("profiles")
}

/// Base configuration directory
fn dirs_path() -> PathBuf {
    if let Ok(dir) = std::env::var("FACECAM_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config").join("facecam")
}

/// Load a profile by name
pub fn load_profile(name: &str) -> Result<Profile> {
    let path = profiles_dir().join(format!("{}.toml", name));
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profile: {}", path.display()))?;
    let profile: Profile =
        toml::from_str(&content).with_context(|| format!("Failed to parse profile: {}", name))?;
    Ok(profile)
}

/// Save a profile
pub fn save_profile(profile: &Profile) -> Result<PathBuf> {
    let dir = profiles_dir();
    fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{}.toml", profile.name));
    let content = toml::to_string_pretty(profile)?;
    fs::write(&path, content)?;

    info!(name = %profile.name, path = %path.display(), "Profile saved");
    Ok(path)
}

/// List all available profiles
pub fn list_profiles() -> Result<Vec<String>> {
    let dir = profiles_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            if let Some(stem) = path.file_stem() {
                profiles.push(stem.to_string_lossy().to_string());
            }
        }
    }

    profiles.sort();
    Ok(profiles)
}

/// Delete a profile
pub fn delete_profile(name: &str) -> Result<()> {
    let path = profiles_dir().join(format!("{}.toml", name));
    if path.exists() {
        fs::remove_file(&path)?;
        info!(name, "Profile deleted");
    }
    Ok(())
}

/// Create default profiles based on empirical device testing.
///
/// Confirmed control ranges (firmware 4.09):
///   brightness: 0-255 (default 128)
///   contrast: 0-10 (default 3)
///   saturation: 0-63 (default 35)
///   sharpness: 0-4 (default 2)
///   white_balance_temperature_auto: bool (default 1)
///   white_balance_temperature: 2800-12500 step 100 (default 5000)
///   power_line_frequency: menu 0=off,1=50Hz,2=60Hz (default 2)
///   auto_exposure: menu 0=auto,2=shutter_priority (default 0)
///   exposure_absolute: 1-2500 in 100us units (default 156 = 1/64s)
///   zoom_absolute: 1-31 (default 1)
///
/// Actual pixel format is UYVY (not YUYV). MJPEG also available.
pub fn create_default_profiles() -> Result<()> {
    let defaults = vec![
        Profile {
            name: "default".to_string(),
            description: "Default settings — 1080p30, auto exposure, auto white balance"
                .to_string(),
            video_mode: Some(ProfileVideoMode {
                width: 1920,
                height: 1080,
                fps: 30,
                format: "UYVY".to_string(),
            }),
            controls: HashMap::from([
                ("brightness".to_string(), 128),
                ("contrast".to_string(), 3),
                ("saturation".to_string(), 35),
                ("sharpness".to_string(), 2),
                ("white_balance_temperature_auto".to_string(), 1),
                ("auto_exposure".to_string(), 0),  // Auto mode
                ("power_line_frequency".to_string(), 2), // 60Hz
            ]),
        },
        Profile {
            name: "streaming".to_string(),
            description: "Optimized for live streaming — 1080p60 MJPEG, balanced settings".to_string(),
            video_mode: Some(ProfileVideoMode {
                width: 1920,
                height: 1080,
                fps: 60,
                format: "MJPG".to_string(),
            }),
            controls: HashMap::from([
                ("brightness".to_string(), 140),
                ("contrast".to_string(), 4),
                ("saturation".to_string(), 40),
                ("sharpness".to_string(), 2),
                ("white_balance_temperature_auto".to_string(), 1),
                ("auto_exposure".to_string(), 0),
                ("power_line_frequency".to_string(), 2),
            ]),
        },
        Profile {
            name: "lowlight".to_string(),
            description: "Low light conditions — 720p30, higher brightness, manual exposure"
                .to_string(),
            video_mode: Some(ProfileVideoMode {
                width: 1280,
                height: 720,
                fps: 30,
                format: "UYVY".to_string(),
            }),
            controls: HashMap::from([
                ("brightness".to_string(), 200),
                ("contrast".to_string(), 2),
                ("saturation".to_string(), 30),
                ("sharpness".to_string(), 1),
                ("white_balance_temperature_auto".to_string(), 1),
                ("auto_exposure".to_string(), 2), // Shutter priority
                ("exposure_absolute".to_string(), 500), // ~1/20s for low light
            ]),
        },
        Profile {
            name: "meeting".to_string(),
            description: "Video conferencing — 720p30 MJPEG, bandwidth-friendly".to_string(),
            video_mode: Some(ProfileVideoMode {
                width: 1280,
                height: 720,
                fps: 30,
                format: "MJPG".to_string(),
            }),
            controls: HashMap::from([
                ("brightness".to_string(), 140),
                ("contrast".to_string(), 3),
                ("saturation".to_string(), 35),
                ("sharpness".to_string(), 2),
                ("white_balance_temperature_auto".to_string(), 1),
                ("auto_exposure".to_string(), 0),
            ]),
        },
    ];

    for profile in &defaults {
        let path = profiles_dir().join(format!("{}.toml", profile.name));
        if !path.exists() {
            save_profile(profile)?;
        }
    }

    Ok(())
}
