# facecam-harness

Automated compatibility and stability testing. Produces machine-readable reports.

## Usage

```bash
facecam-harness [--device /dev/video0] [--json] [-v] <command>
```

## Commands

### full

Runs the complete test suite:

```bash
$ facecam-harness full

  Running: device_detection ...
  [PASS] device_detection (45ms)
  Running: format_enumeration ...
  [PASS] format_enumeration (12ms)
  Running: control_enumeration ...
  [PASS] control_enumeration (8ms)
  Running: format_negotiation ...
  [PASS] format_negotiation (23ms)
  Running: open_close_cycles ...
  [PASS] open_close_cycles (1204ms)
  Running: control_roundtrip ...
  [PASS] control_roundtrip (15ms)
  Running: usb_topology ...
  [PASS] usb_topology (3ms)
  Running: kernel_modules ...
  [PASS] kernel_modules (1ms)

  8/8 passed, 0 failed
```

### Individual Tests

```bash
facecam-harness formats                          # Format enumeration
facecam-harness open-close --cycles 50           # Open/close stability
facecam-harness controls                         # Control roundtrip
facecam-harness stream-stability --duration 300  # 5-minute soak test
facecam-harness recovery                         # USB reset test
```

### report

List and view previous test reports:

```bash
facecam-harness report
```

Reports are saved to `~/.local/share/facecam/harness/harness-<timestamp>.json`.

## Test Matrix

The harness validates:

| Test | What it checks |
|------|---------------|
| `device_detection` | USB enumeration finds Facecam with correct PID |
| `format_enumeration` | V4L2 formats are listed, bogus ones flagged |
| `control_enumeration` | All expected controls are present |
| `format_negotiation` | Each format can be set via VIDIOC_S_FMT |
| `open_close_cycles` | Device survives N open/close cycles |
| `control_roundtrip` | Set/get control values match |
| `usb_topology` | Device is on USB 3.0+ |
| `kernel_modules` | uvcvideo loaded, v4l2loopback status |
| `stream_stability` | Device remains responsive over time |
| `usb_recovery` | USB reset mechanism works |

## CI Integration

```bash
facecam-harness --json full > report.json
# Check exit code: 0 = all passed, 1 = failures
```
