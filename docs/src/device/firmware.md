# Firmware Versions

## Version History

| Firmware | Key Changes | Linux Impact |
|----------|------------|--------------|
| 2.00 | Initial release | Only uncompressed formats; format bugs |
| 2.52 | Fixed settings save bugs | Minor |
| 3.00 | Bulk/iso transfer mode; improved ISP | Transfer mode selectable |
| **4.00+** | **MJPEG format added** | Chromium compatibility fixed |
| **4.09** | Latest; minor refinements | Confirmed working, recommended |

## Detecting Firmware Version

```bash
facecam-probe detect    # Shows firmware field
facecam-probe topology  # Shows bcdDevice raw value
```

The firmware version comes from the USB `bcdDevice` descriptor. `0x0409` = firmware 4.09.

## Firmware Update

> Firmware updates can **only** be performed via Camera Hub on Windows or macOS. There is no Linux-native update mechanism.

If you're on firmware < 4.00:
1. Borrow a Windows or Mac machine
2. Install [Camera Hub](https://www.elgato.com/downloads)
3. Connect the Facecam and follow the update prompt
4. Verify on Linux: `facecam-probe detect` should show 4.09

## Firmware-Gated Features

| Feature | Minimum Firmware |
|---------|-----------------|
| MJPEG format | 4.00 |
| Bulk/iso transfer mode selection | 3.00 |
| Reliable settings save | 2.52 |
