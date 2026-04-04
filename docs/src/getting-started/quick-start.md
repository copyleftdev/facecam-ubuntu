# Quick Start

## 1. Plug In the Camera

Connect the Elgato Facecam to a **USB 3.0 port** (blue port, USB-C, or Thunderbolt). USB 2.0 will not work — the camera enters a fallback mode with PID `0x0077` and refuses to expose any video interface.

## 2. Verify Detection

```bash
facecam-probe detect
```

Expected output:
```
Device:     Elgato Facecam (PID 0x0078)
Firmware:   4.09
USB:        bus 10 addr 2 (SuperSpeed (5 Gbps))
V4L2:       /dev/video0
Card:       Elgato Facecam: Elgato Facecam
```

If you see `USB2 FALLBACK`, move the cable to a different port or try a different cable.

## 3. Check What the Camera Offers

```bash
facecam-probe formats     # List pixel formats and resolutions
facecam-probe controls    # List all V4L2 controls with ranges
facecam-probe quirks      # Show known device quirks
```

## 4. See It Live

```bash
facecam-visual --resolution 720
```

A window opens with your live camera feed plus diagnostic overlays. Press <kbd>W</kbd> for zebra stripes, <kbd>E</kbd> for focus peaking.

## 5. Start the Daemon

The daemon captures from the physical camera and outputs to a v4l2loopback virtual camera that all apps can use:

```bash
# Load v4l2loopback if not already loaded
sudo modprobe v4l2loopback video_nr=10 card_label="Facecam Normalized" exclusive_caps=1

# Start the daemon
sudo systemctl start facecam-daemon

# Check status
facecam-ctl status
```

## 6. Use in Applications

Open OBS, Chrome, Zoom, or any video app and select **"Facecam Normalized"** (`/dev/video10`) as your camera. This virtual camera is stable across open/close cycles and works with all applications including Chromium-based browsers.

## 7. Adjust Settings

```bash
facecam-ctl profile list                    # See available profiles
facecam-ctl profile apply streaming         # Apply streaming preset
facecam-ctl control set brightness 150      # Tweak individual controls
facecam-ctl control list                    # See all current values
```
