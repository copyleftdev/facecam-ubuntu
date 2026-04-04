# Hardware Requirements

## USB 3.0 is Mandatory

The Elgato Facecam requires USB 3.0 SuperSpeed (5 Gbps). On USB 2.0, the camera presents a different product ID (`0x0077` instead of `0x0078`) with the string `USB3-REQUIRED-FOR-FACECAM` and exposes no video interface.

**UYVY 1080p60 requires ~249 MB/s** — over 60% of USB 2.0's theoretical maximum and well beyond its practical throughput.

### How to Identify USB 3.0 Ports

- **Blue USB-A ports** are USB 3.0+
- **USB-C ports** are usually USB 3.0+ (but verify with `lsusb -t`)
- **Thunderbolt ports** support USB 3.0+

### Verify After Connecting

```bash
lsusb | grep 0fd9
```

- `0fd9:0078` on an **even-numbered bus** (002, 004, 006...) = USB 3.0, correct
- `0fd9:0077` on an **odd-numbered bus** = USB 2.0 fallback, move the cable

Or use the probe:
```bash
facecam-probe detect
```

## Cable Quality Matters

A USB-C to USB-C cable must be USB 3.0 rated. Common failure mode: a USB 2.0 cable causes the camera to fall back to PID `0x0077` even when plugged into a 3.0 port. If you see the fallback, **try a different cable first**.

## USB Topology

Avoid sharing USB controllers with other high-bandwidth devices. UYVY 1080p60 at ~249 MB/s leaves little headroom on a shared USB 3.0 controller.

```bash
facecam-probe topology    # Shows sysfs path, speed, bus info
lsusb -t                 # Full USB tree
```

Thunderbolt docks are a known source of instability due to bandwidth sharing.

## Bandwidth by Mode

| Mode | Format | Bandwidth |
|------|--------|-----------|
| 1080p60 | UYVY | ~249 MB/s |
| 1080p30 | UYVY | ~124 MB/s |
| 720p60 | UYVY | ~111 MB/s |
| 720p30 | UYVY | ~55 MB/s |
| 1080p60 | MJPEG | ~5-15 MB/s (variable) |
| 720p30 | MJPEG | ~2-5 MB/s (variable) |

MJPEG mode drastically reduces bandwidth and is recommended when sharing USB controllers.
