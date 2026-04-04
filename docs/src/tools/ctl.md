# facecam-ctl

Control CLI for the daemon. Communicates via Unix domain socket.

## Commands

### status

```bash
$ facecam-ctl status

State:          streaming
Health:         healthy
Uptime:         3600s
Connected:      true
Mode:           1280x720 @ 30.0 fps (UYVY)
Frames:         captured=108000 written=107998 dropped=2
Recoveries:     0
Profile:        streaming
```

### control

```bash
facecam-ctl control list                     # All controls with values
facecam-ctl control get brightness           # Single control
facecam-ctl control set brightness 150       # Set a control
facecam-ctl control set contrast 5
facecam-ctl control set zoom 10
```

### profile

```bash
facecam-ctl profile list                     # Available profiles
facecam-ctl profile show streaming           # Profile details
facecam-ctl profile apply streaming          # Apply a profile
facecam-ctl profile init                     # Create default profiles
```

### diagnostics

Exports a full diagnostics bundle (JSON) for remote debugging:

```bash
$ facecam-ctl diagnostics
Diagnostics exported to: ~/.local/share/facecam/diagnostics/facecam-diag-20260404-185406.json
```

### Operational Commands

```bash
facecam-ctl reset      # Force USB reset
facecam-ctl restart    # Restart the capture pipeline
facecam-ctl shutdown   # Stop the daemon
```

## JSON Output

All commands support `--json`:

```bash
facecam-ctl --json status | jq .state
facecam-ctl --json control list | jq '.[].name'
```
