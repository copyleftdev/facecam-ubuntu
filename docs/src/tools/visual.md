# facecam-visual

Live visual diagnostic tool with broadcast-grade analysis overlays.

## Usage

```bash
facecam-visual [--device /dev/video0] [--resolution 720] [--fps 30] [--mjpeg]
```

## Window Layout

```
+---------------------------------------+
|          LIVE CAMERA FEED             |
|    [30 FPS]              [ZEBRA]     |
|                          [FOCUS]     |
|                          [ A/B ]     |
+=======================================+
| WAVEFORM MONITOR  | RGB HISTOGRAM    |
+---------------------------------------+
| 30.0fps 33.3ms  Brt:128  Con:5 ...  |
+---------------------------------------+
| Frame Timing Waterfall               |
+---------------------------------------+
```

## Keyboard Controls

### Camera Controls

| Key | Action |
|-----|--------|
| <kbd>+</kbd> / <kbd>-</kbd> | Brightness up/down (step 10) |
| <kbd>[</kbd> / <kbd>]</kbd> | Contrast up/down (step 1) |
| <kbd>Z</kbd> / <kbd>X</kbd> | Zoom in/out (step 1) |

### Diagnostic Overlays

| Key | Feature | Description |
|-----|---------|-------------|
| <kbd>W</kbd> | Zebra stripes | Red diagonal hatching on pixels with luma > 235 (overexposure) |
| <kbd>E</kbd> | Focus peaking | Magenta dots on sharp edges (Sobel edge detection) |
| <kbd>A</kbd> | A/B capture | Freezes current frame as reference (left side) |
| <kbd>D</kbd> | A/B clear | Removes the reference frame |
| <kbd><</kbd> / <kbd>></kbd> | A/B split | Moves the comparison split line |

### General

| Key | Action |
|-----|--------|
| <kbd>S</kbd> | Save snapshot (PPM format with overlays) |
| <kbd>R</kbd> | Force USB reset |
| <kbd>Space</kbd> | Pause/unpause |
| <kbd>F</kbd> | Toggle help text |
| <kbd>Q</kbd> / <kbd>Esc</kbd> | Quit |

## Analysis Features

### Waveform Monitor (bottom-left)

Plots luma distribution per video column. Standard broadcast tool for checking exposure:
- **Green** = safe range (16-235 IRE)
- **Yellow** = hot highlights (>200)
- **Red** = clipping (>235)
- **Blue** = crushed blacks (<16)
- Dashed reference lines at 0 IRE and 100 IRE

### RGB Histogram (bottom-right)

Per-channel pixel value distribution with additive blending:
- Red, green, blue curves overlaid
- Left-edge blue bar = crushed blacks (>2% of pixels at 0)
- Right-edge red bar = blown highlights (>2% of pixels at 255)

### Frame Timing Waterfall (bottom strip)

Scrolling timeline where each column = one frame:
- **Green** = delivered on time (<110% of target)
- **Yellow** = late (110-150% of target)
- **Orange** = very late (150-200%)
- **Red** = severely delayed or dropped (>200%)
- Dashed reference line at target frame time

### Zebra Stripes

Industry-standard overexposure indicator. Red diagonal hatching overlaid on any pixel where luma exceeds 235. The pattern animates to distinguish it from image content.

### Focus Peaking

Sobel edge detection with configurable sensitivity. Magenta dots mark sharp edges in the image. Since the Facecam has fixed focus, this helps verify subject distance is in the lens sweet spot.

### A/B Comparison

Capture a reference frame with <kbd>A</kbd>, then adjust controls. The window splits: left = reference, right = live. Move the split with <kbd><</kbd>/<kbd>></kbd>. Clear with <kbd>D</kbd>.

## Session Summary

On exit, prints aggregate statistics:

```
Session summary:
  Total frames: 1847
  Dropped:      0
  Avg FPS:      30.0
  Avg frame:    33.3ms
  Jitter:       0.42ms
  Uptime:       61.6s
```
