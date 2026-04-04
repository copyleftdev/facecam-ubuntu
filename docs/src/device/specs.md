# Elgato Facecam Specs

## Hardware

| Field | Value |
|-------|-------|
| Vendor ID | `0x0fd9` (Elgato Systems GmbH) |
| Product ID | `0x0078` (normal) / `0x0077` (USB2 fallback) |
| UVC Version | 1.10 |
| Sensor | Sony STARVIS CMOS |
| Lens | Elgato Prime Lens, all-glass, f/2.4, 24mm equivalent |
| Focus | Fixed (no autofocus) |
| Max Resolution | 1920x1080 |
| FOV | 82 degrees diagonal |
| USB | 3.0 SuperSpeed (mandatory) |
| Power | Bus-powered, 304mA max |
| Interfaces | UVC (video) + HID (proprietary controls) |

## USB Descriptor Summary

From `lsusb -v`:

- **Configuration**: `USB-3.0`
- **Camera Terminal**: Auto-Exposure Mode, Exposure Time Absolute, Zoom Absolute
- **Processing Unit**: Brightness, Contrast, Saturation, Sharpness, White Balance Temperature, Power Line Frequency, WB Auto
- **Extension Unit**: GUID `{a8e5782b-36e6-4fa1-87f8-83e32b323124}`, 9 proprietary controls (noise reduction, metering mode, save-to-flash, etc.)
- **HID Interface**: Endpoint 0x89, for Camera Hub protocol

## Video Formats (Firmware 4.09)

| Format | Resolutions | Frame Rates |
|--------|------------|-------------|
| UYVY 4:2:2 | 1920x1080, 1280x720, 960x540 | 60, 30 |
| MJPEG | 1920x1080, 1280x720, 960x540 | 60, 30 |

## Still Image Capture

The USB descriptor advertises still image capture:
- **UYVY**: 3840x2160 (4K)
- **MJPEG**: 4128x3096, 4128x2322

These are available via UVC still image capture but not through standard V4L2 streaming.

## Color Space

- Color Primaries: BT.709 / sRGB
- Transfer Function: BT.709
- Matrix Coefficients: SMPTE 170M (BT.601)
