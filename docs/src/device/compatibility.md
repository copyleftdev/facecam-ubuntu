# App Compatibility

## Via Normalized Virtual Camera (Recommended)

When using the daemon with v4l2loopback (`/dev/video10`), **all applications work**:

| Application | Status | Notes |
|-------------|--------|-------|
| OBS Studio | Works | Select "Facecam Normalized" |
| Chrome/Chromium | Works | `exclusive_caps=1` required |
| Firefox | Works | |
| Zoom | Works | |
| Google Meet | Works | Via Chrome |
| Microsoft Teams | Works | Via Chrome |
| Slack | Works | Electron/Chromium |
| Discord | Works | |
| Cheese | Works | |
| mpv | Works | `mpv av://v4l2:/dev/video10` |

## Direct Device Access (No Daemon)

Without the normalization pipeline, compatibility depends on firmware version:

| Application | Firmware < 4.00 | Firmware 4.00+ |
|-------------|-----------------|----------------|
| OBS Studio | Works (UYVY) | Works |
| Firefox | Works | Works |
| Chrome/Chromium | **FAILS** | Works (MJPEG) |
| Electron apps | **FAILS** | Works |
| Cheese | Works | Works |

### Why Chromium Fails

Chromium's camera enumeration rejects V4L2 devices that report both `V4L2_CAP_VIDEO_CAPTURE` and `V4L2_CAP_VIDEO_OUTPUT` capabilities. Additionally, pre-4.00 firmware only offers uncompressed formats which Chromium cannot negotiate.

The v4l2loopback `exclusive_caps=1` parameter solves this by making the virtual device report only `CAPTURE` capability to consumers.

## PipeWire / Wayland

On modern Wayland desktops, some applications access cameras through PipeWire's xdg-desktop-portal Camera interface rather than V4L2 directly. The v4l2loopback device is visible through PipeWire's V4L2 backend, so the normalization pipeline remains compatible.

```bash
# Verify PipeWire sees the virtual camera
pw-cli list-objects | grep -i facecam
```
