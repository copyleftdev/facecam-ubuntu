# Elgato Facecam on Ubuntu — Technical Research Memo

**Date:** 2026-04-04
**Target:** Ubuntu 25.10 (kernel 6.17.0-20-generic)
**Device:** Elgato Facecam (USB VID `0x0fd9`, PID `0x0078`)

---

## 1. Device Identity

| Field | Value |
|-------|-------|
| Vendor ID | `0x0fd9` (Elgato Systems GmbH) |
| Product ID | `0x0078` |
| UVC Version | 1.10 |
| USB Requirement | USB 3.0 (SuperSpeed) — will not operate on USB 2.0 |
| Sensor | Sony STARVIS CMOS |
| Lens | Elgato Prime Lens, all-glass, f/2.4, 24mm equivalent |
| Focus | Fixed (no autofocus) |
| Max Resolution | 1920×1080 (Full HD) |
| FOV | 82° diagonal |
| Secondary Interface | USB HID (vendor-specific controls, flash storage protocol) |

Related Elgato PIDs for reference:
- `0x0079` — Facecam Pro
- `0x0093` / `0x0094` — Facecam MK.2
- `0x0066` — Cam Link 4K

## 2. Advertised vs. Actual Format Support

### Pre-firmware 4.03

The device advertises three uncompressed formats via USB descriptors:

| Format | Advertised | Actually Works |
|--------|-----------|----------------|
| YUYV 4:2:2 | Yes | **Yes** |
| NV12 (Y/CbCr 4:2:0) | Yes | **No** — green/garbage frames |
| YU12 (Planar YUV 4:2:0) | Yes | **No** — green/garbage frames |

This is a firmware-level descriptor bug. The kernel has no quirk entry for PID `0x0078` (unlike the Cam Link 4K `0x0066` which got `UVC_QUIRK_FIX_FORMAT_INDEX` in kernel 5.14).

### Firmware 4.03+

MJPEG added as a fourth format. This resolves Chromium compatibility since Chromium can negotiate MJPEG.

### Bandwidth

YUYV at 1080p60: `1920 × 1080 × 2 bytes × 60 fps ≈ 249 MB/s`. This consumes a substantial fraction of USB 3.0 SuperSpeed bandwidth (~400 MB/s practical), making USB topology a critical variable.

### Pixel Format Ambiguity

Community workarounds use `uyvy422` as the ffmpeg input format despite v4l2 reporting `yuyv422`. The actual wire format may be UYVY. This must be verified empirically via frame inspection.

## 3. Supported Frame Modes

All formats advertise the following resolutions/framerates:

| Resolution | Frame Rates |
|-----------|-------------|
| 1920×1080 | 60, 30 |
| 1280×720  | 60, 30 |
| 960×540   | 60, 30 |

## 4. V4L2 Controls

### Working via standard v4l2:

| Control | Type | Range | Default |
|---------|------|-------|---------|
| `brightness` | int | varies | ~180 |
| `contrast` | int | varies | ~3 |
| `saturation` | int | varies | ~35 |
| `sharpness` | int | varies | ~1 |
| `white_balance_temperature_auto` | bool | 0-1 | 1 |
| `white_balance_temperature` | int | varies | — |
| `power_line_frequency` | menu | 0=off,1=50Hz,2=60Hz | — |
| `exposure_auto` | menu | 1=manual,3=aperture_priority | 3 |
| `exposure_absolute` | int | varies | — |
| `zoom_absolute` | int | varies | ~5 |

### NOT available via v4l2 (requires proprietary HID protocol):

- Noise reduction toggle
- Save settings to on-device flash
- Shutter speed (separate from exposure)
- ISO sensitivity (separate from gain)
- USB transfer mode (bulk/isochronous)
- Firmware update

The proprietary controls are implemented through UVC Extension Units and/or HID feature reports. The Camera Hub (Windows/Mac) communicates via both UVC standard controls and the vendor-specific HID interface. No public documentation exists for this protocol.

## 5. Firmware Versions and Behavior

| Firmware | Key Changes | Linux Impact |
|----------|------------|--------------|
| 2.00 | Initial release | Only uncompressed; format bugs present |
| 2.52 | Fixed settings save bugs | Minor |
| 3.00 | Bulk/iso transfer mode; improved ISP | Transfer mode selectable via Camera Hub |
| **4.03** | **MJPEG format added** | **Critical: fixes Chromium compatibility** |
| 4.09 | Latest; minor refinements | Same MJPEG support |

**Firmware updates require Windows or macOS** — there is no Linux-native update mechanism. This is an unavoidable external dependency.

## 6. Critical Bug: Open/Close Cycle Lockup

**The most impactful Linux-specific issue.** After the first application closes the device, subsequent opens fail until a USB reset is performed.

### Symptoms:
- Camera works for first consumer application
- After that application closes, device returns `EBUSY` or fails to initialize stream
- Physical unplug/replug restores functionality
- Approximately 50% startup failure rate reported even on first open

### Software USB Reset Workaround:
```
# Find device in sysfs by VID:PID
DEVPATH=$(find /sys/bus/usb/devices/ -name "idVendor" -exec grep -l "0fd9" {} \; | head -1 | xargs dirname)
# Deauthorize and reauthorize
echo 0 > "$DEVPATH/authorized"
sleep 0.5
echo 1 > "$DEVPATH/authorized"
```
This forces kernel re-enumeration without physical unplug.

### Mitigation Strategy:
The v4l2loopback approach insulates consumers from this bug. A single long-lived daemon process owns the physical device and writes to a virtual camera. Consumer applications open/close the virtual camera without affecting the physical device.

## 7. Application Compatibility Matrix

### Direct device access:

| Application | Firmware < 4.03 | Firmware ≥ 4.03 |
|-------------|-----------------|-----------------|
| OBS Studio | Works (YUYV) | Works |
| Firefox | Works | Works |
| Chromium/Chrome | **FAILS** | Works (MJPEG) |
| Electron apps (Slack, Signal, Discord desktop) | **FAILS** | Likely works |
| Google Meet (via Chrome) | **FAILS** | Works |
| Cheese | Works | Works |

### Via v4l2loopback virtual camera:
**All applications work** regardless of firmware version, provided:
- `exclusive_caps=1` is set (mandatory for Chromium)
- Producer (daemon) is actively streaming before consumer opens

### Known Chromium Heuristic:
Chromium rejects devices reporting both `V4L2_CAP_VIDEO_CAPTURE` and `V4L2_CAP_VIDEO_OUTPUT`. The `exclusive_caps=1` parameter makes v4l2loopback report only the appropriate capability based on opener role.

## 8. USB Topology Sensitivity

- USB 3.0 is mandatory — device will not enumerate on USB 2.0
- Hubs and shared controllers can cause bandwidth starvation at ~249 MB/s
- Thunderbolt docks are a known source of instability
- Transfer mode (bulk vs. isochronous) is stored in device flash, set via Camera Hub
- Default on Linux: bulk transfer
- Isochronous has been reported to resolve freezing on some systems

## 9. Kernel Module Quirks

The `uvcvideo` module supports a `quirks` parameter (uint bitmask) that can be applied globally:

```
modprobe uvcvideo quirks=0x00000200
```

Relevant flags for potential Facecam use:
- `UVC_QUIRK_PROBE_MINMAX` (0x02) — probe min/max during format negotiation
- `UVC_QUIRK_PROBE_DEF` (0x100) — use probe default values
- `UVC_QUIRK_RESTRICT_FRAME_RATE` (0x200) — restrict frame rate selection
- `UVC_QUIRK_RESTORE_CTRLS_ON_INIT` (0x400) — restore controls on init
- `UVC_QUIRK_DISABLE_AUTOSUSPEND` (0x8000) — disable autosuspend

**Note:** These apply to ALL UVC cameras on the system. Per-device quirks require either a kernel patch to `uvc_driver.c` or runtime workarounds in the daemon.

## 10. Existing Community Projects

| Project | Approach | Limitations |
|---------|----------|-------------|
| ubuntu-elgato-facecam (Aaronontheweb) | Python + FFmpeg + v4l2loopback | Shell-heavy, no device recovery |
| ArchLinux gist (catrielmuller) | Shell + systemd + ffmpeg | Fragile, ~50% startup failure |
| LinuxFaceCam (fu2re) | Shell daemon + v4l2loopback | Minimal, placeholder video |
| cameractrls (soyersoyer) | Python V4L2 control GUI | No Elgato extension unit support |

None provide: deterministic recovery, structured diagnostics, quirk registry, profile persistence, or comprehensive format validation.

## 11. Operational Hazards

1. **uvcdynctrl log flooding**: On Ubuntu, v4l2loopback usage can cause `/var/log/uvcdynctrl-udev.log` to grow unbounded.
2. **DKMS build failures**: v4l2loopback < 0.15.0 fails on kernel ≥ 6.8 due to `strlcpy()` removal.
3. **PipeWire dual-enumeration**: PipeWire may expose the same camera via both V4L2 and libcamera backends, creating duplicate entries.
4. **Device number instability**: `/dev/videoN` numbers are not stable across reboots. Must use udev symlinks.

## 12. Architecture Implications

1. **Single-producer daemon** must own the physical device to isolate consumers from open/close bugs
2. **v4l2loopback with `exclusive_caps=1`** is non-negotiable for Chromium compatibility
3. **Format probing** must validate actual frame delivery, not trust USB descriptors
4. **USB sysfs reset** is the recovery mechanism — must be automated in the daemon
5. **Firmware version detection** (via `bcdDevice` in USB descriptor) gates which workarounds are needed
6. **USB topology audit** should warn about hubs and shared controllers
7. **Profile persistence** must use filesystem (TOML/JSON), not device flash (proprietary protocol)
8. **Structured logging** (JSON) enables remote debugging via support bundle export

## 13. Recommended Rust Crate Stack

| Purpose | Crate | Notes |
|---------|-------|-------|
| V4L2 capture/output | `v4l` | Most mature V4L2 Rust crate |
| v4l2loopback management | `v4l2loopback` | Device create/destroy only |
| USB fingerprinting | `rusb` | VID:PID, firmware, topology |
| Async runtime | `tokio` | For daemon event loop |
| CLI framework | `clap` | For probe, ctl, harness CLIs |
| Serialization | `serde` + `toml` | Profile/config persistence |
| Logging | `tracing` + `tracing-subscriber` | Structured JSON logging |
| IPC | Unix domain socket | Daemon ↔ CLI communication |

## 14. Test Matrix

The compatibility harness must validate across these dimensions:

- **Firmware version**: Pre-4.03, 4.03+, 4.09
- **Pixel format**: YUYV, NV12, YU12, MJPEG (where available)
- **Resolution**: 1080p, 720p, 540p
- **Frame rate**: 60fps, 30fps
- **Open/close cycles**: 1, 5, 10, 50
- **Consumer path**: Direct device, v4l2loopback, PipeWire
- **USB topology**: Direct port, hub, dock
- **Kernel version**: 6.8+, 6.11+, 6.17
