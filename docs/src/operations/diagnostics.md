# Diagnostics & Support Bundles

## Exporting a Bundle

```bash
# Via the daemon
facecam-ctl diagnostics

# Standalone (no daemon needed)
facecam-probe diagnostics
```

Bundles are saved to `~/.local/share/facecam/diagnostics/facecam-diag-<timestamp>.json`.

## Bundle Contents

```json
{
  "generated_at": "2026-04-04T18:54:06Z",
  "system": {
    "hostname": "workstation",
    "kernel_version": "Linux 6.17.0-20-generic ...",
    "os_release": "Ubuntu 25.10",
    "ubuntu_version": "25.10",
    "uptime_secs": 97518
  },
  "device": {
    "product": "Facecam",
    "firmware": {"major": 4, "minor": 9},
    "serial": "FW06M1A07449",
    "usb_bus": 10,
    "usb_speed": "Super"
  },
  "daemon_status": {
    "state": "streaming",
    "health": "healthy",
    "frames_captured": 54000,
    "recovery_count": 0
  },
  "controls": [...],
  "kernel_modules": {
    "uvcvideo_loaded": true,
    "uvcvideo_version": "1.1.1",
    "v4l2loopback_loaded": true
  },
  "v4l2_devices": ["/dev/video0", "/dev/video1", "/dev/video10"],
  "config_files": [...]
}
```

## What to Include in Bug Reports

1. The diagnostics JSON bundle
2. Output of `facecam-probe detect --format json`
3. Output of `facecam-probe quirks --format json`
4. `dmesg | grep -i "usb\|uvc\|video"` (last 50 lines)
5. `facecam-harness --json full` report if the camera is connected

## Log Files

The daemon logs to stdout/stderr in JSON format (when running via systemd):

```bash
journalctl -u facecam-daemon --since "1 hour ago" --no-pager
```

Log fields: timestamp, level, message, and structured key-value pairs for every event.
