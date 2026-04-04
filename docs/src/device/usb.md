# USB Behavior

## Dual-PID Enumeration

The Facecam uses different USB Product IDs depending on the port speed:

| Speed | PID | Product String | Video |
|-------|-----|---------------|-------|
| USB 3.0 SuperSpeed | `0x0078` | `Elgato Facecam` | Full UVC |
| USB 2.0 High-Speed | `0x0077` | `USB3-REQUIRED-FOR-FACECAM` | None |

This is a deliberate firmware behavior — the camera refuses to operate on USB 2.0 rather than producing a degraded experience.

## Open/Close Lockup

The most impactful Linux-specific bug. After the first application closes the V4L2 device file descriptor, subsequent opens fail with `EBUSY` or produce no frames.

**Root cause**: Unknown — likely a firmware-side USB endpoint state management issue. The UVC driver's `uvc_video_stop_streaming` may leave the device in a state it can't recover from.

**Workaround**: The normalization daemon keeps the device open continuously. Consumer apps use the v4l2loopback virtual camera.

**Recovery**: USB sysfs `authorized` flag cycle forces kernel re-enumeration:

```bash
# Automated by facecam-ctl reset and the daemon's recovery logic
echo 0 > /sys/bus/usb/devices/10-2/authorized
sleep 0.5
echo 1 > /sys/bus/usb/devices/10-2/authorized
```

## Startup Unreliability

The camera fails to start streaming approximately 50% of the time on first open. The daemon's `retry_with_reset` logic handles this by:

1. Attempting the operation
2. On failure, performing USB reset
3. Waiting for re-enumeration
4. Retrying (up to `max_recovery_attempts` times)

## USB Transfer Modes

Firmware 3.00+ supports two transfer modes:

| Mode | Description | Default |
|------|-------------|---------|
| Bulk | Lower overhead, less error recovery | Linux default |
| Isochronous | Guaranteed bandwidth, better for real-time | Mac default |

The mode is stored in device flash and can only be changed via Camera Hub. Switching to isochronous has been reported to resolve freezing on some systems.

## Sysfs Path

The Facecam's sysfs path follows the pattern:
```
/sys/bus/usb/devices/<bus>-<port>
```

Example: `/sys/bus/usb/devices/10-2` for bus 10, port 2.

Key sysfs files:
```
authorized    — 0/1, controls device binding
idVendor      — 0fd9
idProduct     — 0078
bcdDevice     — 0409 (firmware version)
speed         — 5000 (Mbps)
manufacturer  — Elgato
product       — Elgato Facecam
```
