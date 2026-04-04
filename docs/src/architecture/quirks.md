# Quirk Registry

Every workaround in this project traces to a specific observed device behavior. The quirk registry in `facecam-common::quirks` catalogs these with severity levels and mitigations.

## Active Quirks

| ID | Severity | Summary |
|----|----------|---------|
| `BOGUS_NV12` | Error | NV12 format advertised but produces garbage frames |
| `BOGUS_YU12` | Error | YU12 format advertised but produces garbage frames |
| `OPEN_CLOSE_LOCKUP` | Critical | Device locks up after consumer close/reopen |
| `STARTUP_UNRELIABILITY` | Error | ~50% failure rate on initial stream start |
| `NO_MJPEG_OLD_FW` | Warning | No MJPEG on firmware < 4.00 |
| `CHROMIUM_FORMAT_REJECT` | Error | Chrome rejects devices with both CAPTURE+OUTPUT caps |
| `USB2_FALLBACK_MODE` | Critical | PID changes to 0x0077 on USB 2.0, no video |
| `USB3_REQUIRED` | Critical | Device requires USB 3.0 SuperSpeed |
| `BANDWIDTH_STARVATION` | Warning | Hub sharing causes drops at ~249 MB/s |
| `YUYV_UYVY_AMBIGUITY` | Info | Wire format is UYVY despite some reports of YUYV |

## Quirk Structure

Each quirk contains:

```rust
pub struct Quirk {
    pub id: &'static str,          // Machine-readable identifier
    pub summary: &'static str,     // One-line description
    pub description: &'static str, // Full explanation with observed behavior
    pub affected: QuirkScope,      // What triggers this quirk
    pub severity: QuirkSeverity,   // Info, Warning, Error, Critical
    pub mitigation: QuirkMitigation, // How the system handles it
}
```

## Querying Quirks

```bash
# Show all quirks applicable to connected device
facecam-probe quirks

# JSON output for automation
facecam-probe --format json quirks
```

## Adding New Quirks

When you observe a new device behavior:

1. Document the exact symptoms (error codes, frame data, timing)
2. Identify the trigger conditions (firmware version, format, USB topology)
3. Determine a mitigation
4. Add to `quirks::quirk_registry()` in `crates/facecam-common/src/quirks.rs`

Every quirk must be traceable to a real observation — no speculative entries.
