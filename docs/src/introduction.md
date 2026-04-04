# Facecam Ubuntu Runtime

A Linux-native runtime for the Elgato Facecam on Ubuntu. No official Linux driver exists — this project provides the complete control plane: device probing, a normalization daemon, control CLI, compatibility harness, and a broadcast-grade visual diagnostic tool.

## Why This Exists

The Elgato Facecam is a premium USB webcam that works seamlessly on Windows and macOS via Camera Hub. On Linux, it enumerates as a standard UVC device but exhibits several quirks that break common workflows:

- **Bogus format advertisement** — the device claims to support NV12/YU12 but only UYVY and MJPEG produce valid frames
- **Open/close cycle lockup** — after the first application closes the device, subsequent opens fail until a USB reset
- **~50% startup failure rate** — the camera randomly fails to initialize on first open
- **Chromium incompatibility** — Chrome/Electron apps reject the device without v4l2loopback normalization
- **No Linux control persistence** — settings reset on every plug cycle

This project solves all of these with a Rust-based daemon that owns the physical device, normalizes output through v4l2loopback, and provides deterministic recovery.

## What's Included

| Component | Purpose |
|-----------|---------|
| `facecam-probe` | Detect, fingerprint, and enumerate the camera |
| `facecam-daemon` | Capture from device, output to v4l2loopback virtual camera |
| `facecam-ctl` | Control the daemon: profiles, controls, diagnostics |
| `facecam-harness` | Automated compatibility and stability testing |
| `facecam-visual` | Live viewer with waveform, histogram, zebras, focus peaking |

## Quick Install

```bash
sudo dpkg -i facecam-ubuntu_0.1.0_amd64.deb
sudo apt-get install -f  # pulls v4l2loopback-dkms, v4l-utils
facecam-probe detect
```

## Design Principles

- **Deterministic over optimistic** — every mitigation traces to an observed device behavior
- **Machine-readable everything** — JSON logs, structured diagnostics, typed IPC
- **Ubuntu-first** — tested on 24.04/25.10, kernel 6.8+
- **No fragile shell scripts** — core runtime in Rust, proper error handling
- **Recoverable by design** — USB reset, retry logic, and watchdog built in
