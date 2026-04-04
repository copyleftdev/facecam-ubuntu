# Troubleshooting

## Camera Not Detected

**Symptom**: `facecam-probe detect` shows no devices.

1. Check `lsusb | grep 0fd9` — if nothing, the camera isn't powered
2. Try a different USB port and cable
3. Check `dmesg | tail -20` for USB errors
4. Verify the port is USB 3.0: `lsusb -t`

## USB 2.0 Fallback (PID 0x0077)

**Symptom**: `facecam-probe detect` shows `USB2 FALLBACK — NOT FUNCTIONAL`.

The camera is on a USB 2.0 port or using a USB 2.0 cable.

1. **Try a different cable** — this is the #1 cause
2. Move to a blue USB-A port or USB-C/Thunderbolt port
3. Avoid USB hubs — connect directly to the motherboard

## Black Screen in Applications

**Symptom**: Camera detected but apps show black video.

1. Check format: `v4l2-ctl -d /dev/video0 --get-fmt-video`
2. If using the daemon, verify v4l2loopback is loaded: `ls /dev/video10`
3. Check daemon status: `facecam-ctl status`
4. Try `facecam-visual` to verify the camera produces frames

## EBUSY or Device Locked

**Symptom**: "Device or resource busy" when opening the camera.

Another process has the device open. The open/close lockup quirk may also be triggered.

1. Check: `fuser /dev/video0`
2. Kill the holding process or use the daemon (which owns the device exclusively)
3. Force reset: `facecam-ctl reset` or `facecam-probe` with `--device` pointing to the correct node

## No v4l2loopback Device

**Symptom**: `/dev/video10` doesn't exist.

```bash
sudo modprobe v4l2loopback video_nr=10 card_label="Facecam Normalized" exclusive_caps=1
ls /dev/video10
```

If modprobe fails, install the DKMS package:
```bash
sudo apt-get install v4l2loopback-dkms
```

## Chrome Can't See the Camera

**Symptom**: Chrome/Chromium doesn't list the camera.

1. Use the v4l2loopback virtual camera, not the direct device
2. Verify `exclusive_caps=1` is set: `cat /sys/module/v4l2loopback/parameters/exclusive_caps`
3. The daemon must be actively streaming before Chrome will detect the device
4. Try `chrome://settings/content/camera` to check permissions

## Daemon Enters Failed State

**Symptom**: `facecam-ctl status` shows `state: failed`.

The daemon exhausted its recovery attempts.

```bash
facecam-ctl reset       # Try a USB reset
facecam-ctl restart     # Restart the pipeline
# Or restart the whole service
sudo systemctl restart facecam-daemon
```

Check logs for the root cause:
```bash
journalctl -u facecam-daemon --since "5 min ago"
```

## Poor Frame Rate

**Symptom**: FPS below expected, stuttering.

1. Check USB bandwidth: `facecam-probe topology` — verify SuperSpeed
2. Use MJPEG mode to reduce bandwidth: `facecam-visual --mjpeg`
3. Avoid sharing the USB controller with other devices
4. Check CPU usage — UYVY-to-RGB conversion at 1080p60 needs moderate CPU
