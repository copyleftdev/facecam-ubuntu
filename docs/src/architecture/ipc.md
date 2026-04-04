# IPC Protocol

The daemon and CLI communicate via a Unix domain socket at `$XDG_RUNTIME_DIR/facecam-daemon.sock` (typically `/run/user/1000/facecam-daemon.sock`).

## Wire Format

One JSON object per line, newline-delimited. Client sends a `DaemonCommand`, daemon responds with a `DaemonResponse`.

## Commands

```json
"Status"
{"ApplyProfile": {"name": "streaming"}}
{"SetControl": {"name": "brightness", "value": 150}}
{"GetControl": {"name": "contrast"}}
"GetAllControls"
"ExportDiagnostics"
"ForceReset"
"RestartPipeline"
"Shutdown"
```

## Responses

```json
{"Status": {"state": "streaming", "health": "healthy", "fps": 30.0, ...}}
{"Ok": "Profile 'streaming' applied (7 controls set)"}
{"ControlValue": {"name": "brightness", "value": 150}}
{"Controls": [{"name": "brightness", "id": 9963776, "value": 150, ...}, ...]}
{"DiagnosticsExported": "/home/user/.local/share/facecam/diagnostics/facecam-diag-20260404-185406.json"}
{"Error": "No source device connected"}
```

## Example: Manual Socket Interaction

```bash
# Connect and send a status query
echo '"Status"' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/facecam-daemon.sock
```

## Security

The socket is created with mode `0660`. Users in the `video` group can connect. The daemon requires write access to `/sys/bus/usb/devices/` for USB reset operations.
