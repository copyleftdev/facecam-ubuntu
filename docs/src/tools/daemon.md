# facecam-daemon

The normalization daemon — captures from the physical Facecam and outputs to a v4l2loopback virtual camera.

## Usage

```bash
facecam-daemon [--source /dev/video0] [--sink /dev/video10] \
               [--profile default] [--foreground] [--log-format json|text]
```

## systemd Service

The recommended way to run the daemon:

```bash
sudo systemctl start facecam-daemon
sudo systemctl enable facecam-daemon    # Auto-start on plug
sudo systemctl status facecam-daemon
journalctl -u facecam-daemon -f         # Follow logs
```

The udev rule (`99-facecam.rules`) triggers the service automatically when the Facecam is plugged in.

## Configuration

Config file: `~/.config/facecam/daemon.toml`

```toml
max_recovery_attempts = 5
frame_timeout_ms = 5000

[loopback]
video_nr = 10
card_label = "Facecam Normalized"
max_openers = 10

[logging]
max_events = 1000
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--source` | auto-detect | Physical camera device path |
| `--sink` | `/dev/video10` | v4l2loopback output device |
| `--profile` | `default` | Profile to apply on startup |
| `--foreground` | off | Don't daemonize |
| `--log-format` | `json` | Log format (`json` for production, `text` for debug) |
| `--config` | `~/.config/facecam/daemon.toml` | Config file path |

## Recovery Behavior

The daemon automatically recovers from:
- Device read errors (USB glitch)
- Stream initialization failures (~50% startup bug)
- Device disconnection (unplug/replug)

After `max_recovery_attempts` consecutive failures (default 5), it enters `Failed` state and waits for manual intervention via `facecam-ctl restart` or `facecam-ctl reset`.
