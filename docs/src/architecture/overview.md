# System Overview

## Architecture Diagram

```
                    USB 3.0
                      |
              +-------+-------+
              | Elgato Facecam |  PID 0x0078, UVC 1.10
              |   /dev/video0  |  UYVY + MJPEG
              +-------+-------+
                      |
                      | MMAP streaming (VIDIOC_QBUF/DQBUF)
                      |
              +-------+-------+
              | facecam-daemon |  Single long-lived process
              |                |  Owns the physical device
              |  - Capture     |  Prevents open/close lockup
              |  - Recovery    |  USB reset on failure
              |  - Controls    |  Profile application
              +-------+-------+
                      |
                      | write() frames
                      |
              +-------+-------+
              |  v4l2loopback  |  /dev/video10
              | "Facecam       |  exclusive_caps=1
              |  Normalized"   |  Multiple consumers OK
              +-------+-------+
                      |
         +------------+------------+
         |            |            |
      +--+--+     +--+--+     +--+--+
      | OBS  |    |Chrome|    | Zoom |  Any V4L2 consumer
      +------+    +------+    +------+
```

## Key Design Decisions

### Single-Producer Daemon

The daemon is the **only** process that opens the physical Facecam device. This solves the open/close lockup bug — consumer applications open and close the v4l2loopback device freely without touching the physical hardware.

### v4l2loopback with exclusive_caps

The `exclusive_caps=1` parameter is mandatory. Without it, Chromium-based browsers refuse to use the virtual camera because they reject devices that report both `V4L2_CAP_VIDEO_CAPTURE` and `V4L2_CAP_VIDEO_OUTPUT`.

### USB Reset Recovery

The daemon implements automatic recovery via the sysfs `authorized` flag cycle:

1. Write `0` to `/sys/bus/usb/devices/<dev>/authorized` (deauthorize)
2. Wait 500ms for kernel driver unbind
3. Write `1` to reauthorize (triggers re-enumeration)
4. Wait 1500ms for device stabilization
5. Retry the failed operation

### Profile-Based Control Persistence

V4L2 controls reset on device disconnect. The daemon re-applies the active profile's control values after every recovery cycle, ensuring consistent camera settings.

## Crate Structure

```
facecam-ubuntu/
  crates/
    facecam-common/     Shared library — types, quirks, v4l2, USB, IPC
    facecam-probe/      Device detection and enumeration CLI
    facecam-daemon/     Normalization daemon
    facecam-ctl/        Control CLI (talks to daemon via Unix socket)
    facecam-harness/    Automated compatibility test suite
    facecam-visual/     Live visual diagnostic tool
```

All crates share `facecam-common` for device identification, V4L2 ioctl wrappers, USB enumeration, the quirk registry, profile management, and IPC types.
