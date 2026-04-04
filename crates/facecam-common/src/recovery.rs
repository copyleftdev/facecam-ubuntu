use crate::usb::find_facecam_sysfs_path;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// USB device reset via sysfs authorized flag cycle.
///
/// This is the primary recovery mechanism for the open/close lockup bug.
/// Writing 0 to the `authorized` sysfs file deauthorizes the device,
/// causing the kernel to unbind the driver. Writing 1 reauthorizes it,
/// causing re-enumeration — equivalent to a physical unplug/replug.
pub fn usb_reset_facecam() -> Result<ResetResult> {
    let sysfs_path = find_facecam_sysfs_path()?
        .ok_or_else(|| anyhow::anyhow!("Facecam not found in sysfs"))?;

    usb_reset_device(&sysfs_path)
}

/// Reset a USB device by its sysfs path
pub fn usb_reset_device(sysfs_path: &PathBuf) -> Result<ResetResult> {
    let auth_path = sysfs_path.join("authorized");

    if !auth_path.exists() {
        bail!(
            "sysfs authorized file not found at {}",
            auth_path.display()
        );
    }

    info!(path = %sysfs_path.display(), "Performing USB reset via sysfs");

    // Read current state
    let current = fs::read_to_string(&auth_path)
        .context("Failed to read authorized state")?
        .trim()
        .to_string();

    debug!(current_state = %current, "Current authorized state");

    // Deauthorize
    fs::write(&auth_path, "0").context("Failed to deauthorize USB device")?;
    info!("Device deauthorized, waiting for kernel cleanup");

    // Wait for kernel to unbind driver
    thread::sleep(Duration::from_millis(500));

    // Reauthorize
    fs::write(&auth_path, "1").context("Failed to reauthorize USB device")?;
    info!("Device reauthorized, waiting for re-enumeration");

    // Wait for kernel to re-enumerate
    thread::sleep(Duration::from_millis(1500));

    // Verify device came back
    let new_state = fs::read_to_string(&auth_path)
        .context("Failed to read authorized state after reset")?
        .trim()
        .to_string();

    if new_state != "1" {
        bail!(
            "Device did not come back after reset (authorized = {})",
            new_state
        );
    }

    info!("USB reset complete, device re-enumerated");

    Ok(ResetResult {
        sysfs_path: sysfs_path.clone(),
        success: true,
        previous_state: current,
        new_state,
    })
}

/// Attempt to start a stream with retry-on-failure logic.
///
/// Due to the ~50% startup failure rate, this function:
/// 1. Attempts to open and start the stream
/// 2. On failure, performs a USB reset
/// 3. Waits for the device to re-appear
/// 4. Retries the open
///
/// Returns the number of attempts needed.
pub fn retry_with_reset<F, T>(
    max_attempts: u32,
    operation_name: &str,
    mut operation: F,
) -> Result<(T, u32)>
where
    F: FnMut(u32) -> Result<T>,
{
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        info!(attempt, max_attempts, operation = operation_name, "Attempting operation");

        match operation(attempt) {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt,
                        operation = operation_name,
                        "Operation succeeded after retry"
                    );
                }
                return Ok((result, attempt));
            }
            Err(e) => {
                warn!(
                    attempt,
                    max_attempts,
                    error = %e,
                    operation = operation_name,
                    "Operation failed"
                );
                last_error = Some(e);

                if attempt < max_attempts {
                    info!("Performing USB reset before retry");
                    match usb_reset_facecam() {
                        Ok(reset) => {
                            info!(
                                sysfs = %reset.sysfs_path.display(),
                                "USB reset successful, waiting before retry"
                            );
                            // Extra wait after reset for device stabilization
                            thread::sleep(Duration::from_secs(1));
                        }
                        Err(reset_err) => {
                            error!(error = %reset_err, "USB reset failed");
                        }
                    }
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All {} attempts failed", max_attempts)))
}

/// Wait for the Facecam to appear in sysfs after a reset or plug event
pub fn wait_for_device(timeout: Duration) -> Result<PathBuf> {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(200);

    info!(timeout_ms = timeout.as_millis() as u64, "Waiting for Facecam to appear");

    loop {
        if start.elapsed() > timeout {
            bail!("Timeout waiting for Facecam to appear ({}ms)", timeout.as_millis());
        }

        match find_facecam_sysfs_path() {
            Ok(Some(path)) => {
                info!(path = %path.display(), elapsed_ms = start.elapsed().as_millis() as u64, "Facecam found");
                return Ok(path);
            }
            Ok(None) => {}
            Err(e) => {
                debug!(error = %e, "Error scanning sysfs");
            }
        }

        thread::sleep(poll_interval);
    }
}

#[derive(Debug, Clone)]
pub struct ResetResult {
    pub sysfs_path: PathBuf,
    pub success: bool,
    pub previous_state: String,
    pub new_state: String,
}

/// Check if the Facecam is currently connected and authorized
pub fn check_device_present() -> Result<DevicePresence> {
    match find_facecam_sysfs_path()? {
        Some(path) => {
            let auth_path = path.join("authorized");
            let authorized = if auth_path.exists() {
                fs::read_to_string(&auth_path)?
                    .trim()
                    .parse::<u8>()
                    .unwrap_or(0)
                    == 1
            } else {
                false
            };

            Ok(DevicePresence {
                connected: true,
                authorized,
                sysfs_path: Some(path),
            })
        }
        None => Ok(DevicePresence {
            connected: false,
            authorized: false,
            sysfs_path: None,
        }),
    }
}

#[derive(Debug, Clone)]
pub struct DevicePresence {
    pub connected: bool,
    pub authorized: bool,
    pub sysfs_path: Option<PathBuf>,
}
