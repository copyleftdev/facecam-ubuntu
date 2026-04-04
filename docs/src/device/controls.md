# V4L2 Controls

## Available Controls (Firmware 4.09)

Verified from live device testing:

| Control | CID | Type | Range | Default | Notes |
|---------|-----|------|-------|---------|-------|
| Brightness | `0x00980900` | int | 0-255 | 128 | Processing Unit |
| Contrast | `0x00980901` | int | 0-10 | 3 | Processing Unit |
| Saturation | `0x00980902` | int | 0-63 | 35 | Processing Unit |
| Sharpness | `0x0098091b` | int | 0-4 | 2 | Processing Unit |
| WB Auto | `0x0098090c` | bool | 0-1 | 1 | Disables WB Temp when on |
| WB Temperature | `0x0098091a` | int | 2800-12500 | 5000 | Step 100, Kelvin |
| Power Line Freq | `0x00980918` | menu | 0-2 | 2 | 0=Off, 1=50Hz, 2=60Hz |
| Auto Exposure | `0x009a0901` | menu | 0-3 | 0 | 0=Auto, 2=Shutter Priority |
| Exposure Time | `0x009a0902` | int | 1-2500 | 156 | Units of 100us |
| Zoom Absolute | `0x009a090d` | int | 1-31 | 1 | Digital zoom/crop |

## Exposure Time Values

The exposure time is in units of 100 microseconds:

| Value | Shutter Speed | Use Case |
|-------|--------------|----------|
| 1 | 1/10000s | Bright daylight |
| 10 | 1/1000s | Well-lit room |
| 78 | 1/128s | Indoor |
| 156 | 1/64s | Default |
| 500 | 1/20s | Low light |
| 2500 | 1/4s | Very dark (motion blur) |

## White Balance Temperature

| Kelvin | Light Source |
|--------|-------------|
| 2800 | Candlelight |
| 3200 | Tungsten / warm white |
| 4000 | Fluorescent |
| 5000 | Default / neutral daylight |
| 6500 | Overcast / cool daylight |
| 10000 | Blue sky |
| 12500 | Maximum (very cool) |

## Controls NOT Available via V4L2

These require the proprietary HID protocol (Camera Hub on Windows/Mac):

- Noise reduction (on/off)
- Metering mode (average / center-weighted)
- Save settings to device flash
- USB transfer mode (bulk / isochronous)
- Firmware update

The Extension Unit GUID is `{a8e5782b-36e6-4fa1-87f8-83e32b323124}` with 9 vendor-specific controls.

## Command Line Usage

```bash
# Read controls
facecam-probe controls
facecam-ctl control list
facecam-ctl control get brightness

# Set controls
facecam-ctl control set brightness 150
facecam-ctl control set contrast 5
facecam-ctl control set zoom 10

# Or directly with v4l2-ctl
v4l2-ctl -d /dev/video0 --set-ctrl brightness=150
```
