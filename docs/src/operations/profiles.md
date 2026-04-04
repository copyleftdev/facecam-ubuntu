# Profiles

Profiles store a named set of camera controls and preferred video mode. They persist as TOML files in `~/.config/facecam/profiles/`.

## Default Profiles

```bash
facecam-ctl profile init    # Creates defaults if they don't exist
facecam-ctl profile list
```

| Profile | Resolution | Format | Description |
|---------|-----------|--------|-------------|
| `default` | 1080p30 | UYVY | Factory defaults, auto exposure/WB |
| `streaming` | 1080p60 | MJPEG | Optimized for live streaming |
| `lowlight` | 720p30 | UYVY | Higher brightness, manual exposure |
| `meeting` | 720p30 | MJPEG | Bandwidth-friendly for video calls |

## Profile Format

```toml
name = "streaming"
description = "Optimized for live streaming"

[video_mode]
width = 1920
height = 1080
fps = 60
format = "MJPG"

[controls]
brightness = 140
contrast = 4
saturation = 40
sharpness = 2
white_balance_temperature_auto = 1
auto_exposure = 0
power_line_frequency = 2
```

## Creating Custom Profiles

1. Create a TOML file in `~/.config/facecam/profiles/`:

```bash
cat > ~/.config/facecam/profiles/podcast.toml << 'EOF'
name = "podcast"
description = "Podcast recording — warm tones, shallow zoom"

[video_mode]
width = 1920
height = 1080
fps = 30
format = "UYVY"

[controls]
brightness = 135
contrast = 4
saturation = 45
sharpness = 2
white_balance_temperature_auto = 0
white_balance_temperature = 4200
auto_exposure = 0
zoom = 5
EOF
```

2. Apply it:
```bash
facecam-ctl profile apply podcast
```

## Profile Application

When a profile is applied:
1. Each control in `[controls]` is set via `VIDIOC_S_CTRL`
2. Unknown control names are silently skipped
3. The daemon updates its active profile name in status
4. On recovery (USB reset), the active profile is re-applied automatically
