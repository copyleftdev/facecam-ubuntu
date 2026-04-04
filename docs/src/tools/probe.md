# facecam-probe

Detect, fingerprint, and enumerate the Elgato Facecam. This is the first tool to run when setting up or debugging.

## Usage

```bash
facecam-probe [--format text|json] [-v] <command>
```

## Commands

### detect

Scans USB bus for Elgato devices, enriches with V4L2 info.

```bash
$ facecam-probe detect

Device:     Elgato Facecam (PID 0x0078)
Firmware:   4.09
Serial:     FW06M1A07449
USB:        bus 10 addr 2 (SuperSpeed (5 Gbps))
V4L2:       /dev/video0
Card:       Elgato Facecam: Elgato Facecam
```

### formats

Lists all pixel formats, resolutions, and frame rates. Flags broken formats.

```bash
$ facecam-probe formats

Pixel Formats:
  [0] UYVY - UYVY 4:2:2 [RELIABLE]
  [1] MJPG - Motion-JPEG [RELIABLE]

Video Modes:
  1920x1080 @ 60.0 fps (UYVY) (249 MB/s)
  1280x720 @ 30.0 fps (MJPG)
  ...
```

### controls

Enumerates all V4L2 controls with current values, ranges, and menu items.

```bash
$ facecam-probe controls

  Brightness (0x00980900):
    type=Integer  value=128  range=[0, 255]  step=1  default=128
  Contrast (0x00980901):
    type=Integer  value=5  range=[0, 10]  step=1  default=3
  ...
```

### topology

Shows USB sysfs details: bus, speed, manufacturer, firmware BCD, configuration.

### quirks

Lists all quirks applicable to the detected device and firmware version.

### diagnostics

Collects system info, kernel module status, V4L2 devices, and exports a JSON bundle.

### validate

Attempts to stream each advertised format and verifies actual frame delivery.

## JSON Output

All commands support `--format json` for machine-readable output:

```bash
facecam-probe --format json detect | jq .
```
