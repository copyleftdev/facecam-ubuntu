# Normalization Pipeline

## Frame Flow

```
Physical Device (/dev/video0)
    |
    | VIDIOC_DQBUF (MMAP buffer)
    v
  [Frame in UYVY or MJPEG]
    |
    | Copy to v4l2loopback
    v
Virtual Device (/dev/video10)
    |
    | Consumer opens and reads
    v
  Application (OBS, Chrome, etc.)
```

## Pipeline States

The daemon operates as a state machine:

```
  Idle ──> Probing ──> Starting ──> Streaming
   ^                                   |
   |                                   v
   +──── Failed <──── Recovering <─────+
                         |
                    ShuttingDown
```

| State | Description |
|-------|-------------|
| `Idle` | No device detected, waiting |
| `Probing` | Device found, reading capabilities and controls |
| `Starting` | Format set, MMAP buffers allocated, about to stream |
| `Streaming` | Active frame forwarding |
| `Recovering` | Error detected, performing USB reset |
| `Failed` | Max recovery attempts exceeded |
| `ShuttingDown` | Graceful exit in progress |

## Recovery Logic

On any capture error:

1. Increment recovery counter
2. If counter > `max_recovery_attempts` (default 5): enter `Failed` state
3. Perform USB sysfs reset
4. Wait for device re-enumeration (2s)
5. Re-open device, re-apply format and controls
6. Resume streaming

The `retry_with_reset` helper in `facecam-common::recovery` encapsulates this pattern for any fallible operation.

## Format Selection

The daemon selects the best available format:

1. Load the active profile's preferred format/resolution
2. Filter to formats marked reliable in the quirk registry (UYVY, MJPEG)
3. Match the closest available mode
4. Fall back to the first reliable mode if no match

## Frame Statistics

The daemon tracks per-5-second windows:
- `frames_captured` — total frames dequeued from the device
- `frames_written` — total frames written to v4l2loopback
- `frames_dropped` — frames lost due to sink write failures
- `recovery_count` — number of USB reset cycles performed

These are exposed via the IPC status command and included in diagnostics bundles.
