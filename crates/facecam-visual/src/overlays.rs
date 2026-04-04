/// Broadcast-grade visual analysis overlays for camera diagnostics.
///
/// All overlays operate on the ARGB framebuffer and are composited
/// directly onto the video frame or drawn in dedicated panel areas.

/// Zebra stripes — diagonal hatch pattern over overexposed regions.
/// Standard broadcast threshold: luma > 235 (100 IRE).
pub fn draw_zebras(buf: &mut [u32], w: usize, h: usize, threshold: u8, frame_count: u64) {
    let phase = (frame_count % 8) as usize; // Animate the stripes
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if idx >= buf.len() { break; }
            let pixel = buf[idx];
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            // BT.709 luma
            let luma = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u8;
            if luma > threshold {
                // Draw diagonal stripe pattern
                if ((x + y + phase) / 3) % 2 == 0 {
                    buf[idx] = 0xFF0000; // Red stripe
                }
            }
        }
    }
}

/// Focus peaking — Sobel edge detection with colored overlay.
/// Highlights sharp edges in the image, essential for verifying focus.
pub fn draw_focus_peaking(buf: &mut [u32], w: usize, h: usize, sensitivity: u32, color: u32) {
    // We need luma values — extract first, then overlay
    let mut luma = vec![0i16; w * h];
    for i in 0..w * h {
        if i >= buf.len() { break; }
        let pixel = buf[i];
        let r = ((pixel >> 16) & 0xFF) as i16;
        let g = ((pixel >> 8) & 0xFF) as i16;
        let b = (pixel & 0xFF) as i16;
        luma[i] = (r * 54 + g * 183 + b * 19) >> 8; // Fast BT.601 approx
    }

    // Sobel operator on luma
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let idx = y * w + x;
            // Gx
            let gx = -luma[(y-1)*w + (x-1)] - 2*luma[y*w + (x-1)] - luma[(y+1)*w + (x-1)]
                    + luma[(y-1)*w + (x+1)] + 2*luma[y*w + (x+1)] + luma[(y+1)*w + (x+1)];
            // Gy
            let gy = -luma[(y-1)*w + (x-1)] - 2*luma[(y-1)*w + x] - luma[(y-1)*w + (x+1)]
                    + luma[(y+1)*w + (x-1)] + 2*luma[(y+1)*w + x] + luma[(y+1)*w + (x+1)];
            let magnitude = ((gx.abs() + gy.abs()) as u32) >> 1;
            if magnitude > sensitivity {
                if idx < buf.len() {
                    buf[idx] = color;
                }
            }
        }
    }
}

/// RGB histogram — count pixel values per channel, draw as overlapping curves.
/// Drawn into a specified region of the framebuffer.
pub fn draw_rgb_histogram(
    buf: &mut [u32], buf_w: usize,
    video_buf: &[u32], video_w: usize, video_h: usize,
    panel_x: usize, panel_y: usize, panel_w: usize, panel_h: usize,
) {
    // Count pixel values per channel
    let mut r_hist = [0u32; 256];
    let mut g_hist = [0u32; 256];
    let mut b_hist = [0u32; 256];

    for y in 0..video_h {
        for x in 0..video_w {
            let idx = y * buf_w + x;
            if idx >= video_buf.len() { continue; }
            let pixel = video_buf[idx];
            r_hist[((pixel >> 16) & 0xFF) as usize] += 1;
            g_hist[((pixel >> 8) & 0xFF) as usize] += 1;
            b_hist[(pixel & 0xFF) as usize] += 1;
        }
    }

    // Find max for normalization
    let max_count = r_hist.iter().chain(g_hist.iter()).chain(b_hist.iter())
        .copied().max().unwrap_or(1).max(1);

    // Clear panel area
    fill_rect(buf, buf_w, panel_x, panel_y, panel_w, panel_h, 0x0D0D1A);

    // Draw each channel
    draw_histogram_channel(buf, buf_w, &r_hist, max_count, panel_x, panel_y, panel_w, panel_h, 0xCC0000);
    draw_histogram_channel(buf, buf_w, &g_hist, max_count, panel_x, panel_y, panel_w, panel_h, 0x00CC00);
    draw_histogram_channel(buf, buf_w, &b_hist, max_count, panel_x, panel_y, panel_w, panel_h, 0x0044CC);

    // Draw clipping warnings — red bars at edges if values pile up at 0 or 255
    let total_pixels = (video_w * video_h) as u32;
    let clip_threshold = total_pixels / 50; // >2% is clipping
    if r_hist[255] > clip_threshold || g_hist[255] > clip_threshold || b_hist[255] > clip_threshold {
        // Highlight right edge (overexposure)
        for y in panel_y..panel_y + panel_h {
            let idx = y * buf_w + panel_x + panel_w - 2;
            if idx < buf.len() { buf[idx] = 0xFF0000; }
            if idx + 1 < buf.len() { buf[idx + 1] = 0xFF0000; }
        }
    }
    if r_hist[0] > clip_threshold || g_hist[0] > clip_threshold || b_hist[0] > clip_threshold {
        // Highlight left edge (underexposure / crushed blacks)
        for y in panel_y..panel_y + panel_h {
            let idx = y * buf_w + panel_x;
            if idx < buf.len() { buf[idx] = 0x0000FF; }
            if idx + 1 < buf.len() { buf[idx + 1] = 0x0000FF; }
        }
    }
}

fn draw_histogram_channel(
    buf: &mut [u32], buf_w: usize,
    hist: &[u32; 256], max_count: u32,
    px: usize, py: usize, pw: usize, ph: usize,
    color: u32,
) {
    for i in 0..256 {
        let x = px + (i * pw) / 256;
        let bar_h = ((hist[i] as u64 * ph as u64) / max_count as u64) as usize;
        for row in 0..bar_h.min(ph) {
            let y = py + ph - 1 - row;
            let idx = y * buf_w + x;
            if idx < buf.len() {
                // Additive blend for overlapping channels
                let existing = buf[idx];
                let er = (existing >> 16) & 0xFF;
                let eg = (existing >> 8) & 0xFF;
                let eb = existing & 0xFF;
                let cr = (color >> 16) & 0xFF;
                let cg = (color >> 8) & 0xFF;
                let cb = color & 0xFF;
                buf[idx] = ((er + cr).min(255) << 16) | ((eg + cg).min(255) << 8) | (eb + cb).min(255);
            }
        }
    }
}

/// Waveform monitor — plots luma distribution per column.
/// Each column of the video maps to a column of the waveform.
/// Vertical axis = luma 0 (bottom) to 255 (top).
pub fn draw_waveform(
    buf: &mut [u32], buf_w: usize,
    video_buf: &[u32], video_w: usize, video_h: usize,
    panel_x: usize, panel_y: usize, panel_w: usize, panel_h: usize,
) {
    // Clear panel
    fill_rect(buf, buf_w, panel_x, panel_y, panel_w, panel_h, 0x0D0D1A);

    // Draw reference lines at 0 IRE (16), 100 IRE (235)
    let ire_0_y = panel_y + panel_h - (16 * panel_h / 256);
    let ire_100_y = panel_y + panel_h - (235 * panel_h / 256);
    for x in panel_x..panel_x + panel_w {
        if ire_0_y < buf.len() / buf_w {
            let idx = ire_0_y * buf_w + x;
            if idx < buf.len() && x % 4 == 0 { buf[idx] = 0x333344; }
        }
        if ire_100_y < buf.len() / buf_w {
            let idx = ire_100_y * buf_w + x;
            if idx < buf.len() && x % 4 == 0 { buf[idx] = 0x333344; }
        }
    }

    // For each video column, sample luma values and plot
    let _step = if video_w > panel_w { video_w / panel_w } else { 1 };
    for col in 0..panel_w.min(video_w) {
        let src_col = col * video_w / panel_w;
        // Sample every Nth row for performance
        let row_step = (video_h / 64).max(1);
        for row in (0..video_h).step_by(row_step) {
            let src_idx = row * buf_w + src_col;
            if src_idx >= video_buf.len() { continue; }
            let pixel = video_buf[src_idx];
            let r = ((pixel >> 16) & 0xFF) as u32;
            let g = ((pixel >> 8) & 0xFF) as u32;
            let b = (pixel & 0xFF) as u32;
            let luma = (r * 54 + g * 183 + b * 19) >> 8;

            let plot_y = panel_y + panel_h - 1 - (luma as usize * (panel_h - 1) / 255);
            let plot_x = panel_x + col;
            if plot_y < buf.len() / buf_w && plot_x < buf_w {
                let idx = plot_y * buf_w + plot_x;
                if idx < buf.len() {
                    // Accumulate brightness for density display
                    let existing = buf[idx] & 0xFF;
                    let new_val = (existing + 20).min(255);
                    buf[idx] = (new_val << 16) | (new_val << 8) | new_val;
                }
            }
        }
    }

    // Colorize: green for safe range, yellow for hot, red for clipping
    for y in panel_y..panel_y + panel_h {
        for x in panel_x..panel_x + panel_w {
            let idx = y * buf_w + x;
            if idx >= buf.len() { continue; }
            let brightness = buf[idx] & 0xFF;
            if brightness < 2 { continue; } // Skip empty pixels
            let luma_level = 255 - ((y - panel_y) * 255 / panel_h);
            let color = if luma_level > 235 {
                blend_color(0xFF3333, brightness as u32)  // Red — overexposed
            } else if luma_level > 200 {
                blend_color(0xFFCC00, brightness as u32)  // Yellow — hot
            } else if luma_level < 16 {
                blend_color(0x3333FF, brightness as u32)  // Blue — crushed
            } else {
                blend_color(0x00FF66, brightness as u32)  // Green — safe
            };
            buf[idx] = color;
        }
    }
}

/// Frame timing waterfall — scrolling timeline of frame delivery times.
/// Each column = one frame. Color = timing health. Scrolls left over time.
pub struct TimingWaterfall {
    data: Vec<f64>, // ms per frame
    max_entries: usize,
}

impl TimingWaterfall {
    pub fn new(width: usize) -> Self {
        Self {
            data: Vec::with_capacity(width),
            max_entries: width,
        }
    }

    pub fn push(&mut self, frame_time_ms: f64) {
        if self.data.len() >= self.max_entries {
            self.data.remove(0);
        }
        self.data.push(frame_time_ms);
    }

    pub fn draw(&self, buf: &mut [u32], buf_w: usize,
                px: usize, py: usize, pw: usize, ph: usize,
                target_ms: f64) {
        fill_rect(buf, buf_w, px, py, pw, ph, 0x0D0D1A);

        let start = if self.data.len() > pw { self.data.len() - pw } else { 0 };
        let offset = pw.saturating_sub(self.data.len());

        for (i, &ms) in self.data.iter().skip(start).enumerate() {
            let x = px + offset + i;
            if x >= px + pw { break; }

            let ratio = ms / target_ms;
            let color = if ratio < 1.1 {
                0x00CC66 // Green — on time
            } else if ratio < 1.5 {
                0xCCCC00 // Yellow — late
            } else if ratio < 2.0 {
                0xFF6600 // Orange — very late
            } else {
                0xFF0000 // Red — dropped/stalled
            };

            // Draw bar height proportional to frame time
            let bar_h = ((ms / (target_ms * 3.0)).min(1.0) * ph as f64) as usize;
            for row in 0..bar_h.min(ph) {
                let y = py + ph - 1 - row;
                let idx = y * buf_w + x;
                if idx < buf.len() {
                    buf[idx] = color;
                }
            }
        }

        // Draw target line
        let target_y = py + ph - ((ph as f64 / 3.0) as usize); // target = 1/3 height
        for x in (px..px + pw).step_by(4) {
            let idx = target_y * buf_w + x;
            if idx < buf.len() { buf[idx] = 0x444466; }
        }
    }
}

/// A/B split comparison — stores a reference frame and draws split view.
pub struct ABCompare {
    reference: Option<Vec<u32>>,
    split_pos: f32, // 0.0 = all reference, 1.0 = all live, 0.5 = half/half
}

impl ABCompare {
    pub fn new() -> Self {
        Self { reference: None, split_pos: 0.5 }
    }

    pub fn capture_reference(&mut self, buf: &[u32], w: usize, h: usize) {
        self.reference = Some(buf[..w * h].to_vec());
    }

    pub fn clear_reference(&mut self) {
        self.reference = None;
    }

    pub fn has_reference(&self) -> bool {
        self.reference.is_some()
    }

    pub fn draw_split(&self, buf: &mut [u32], w: usize, h: usize) {
        if let Some(ref reference) = self.reference {
            let split_x = (w as f32 * self.split_pos) as usize;
            // Left side = reference
            for y in 0..h {
                for x in 0..split_x.min(w) {
                    let idx = y * w + x;
                    if idx < buf.len() && idx < reference.len() {
                        buf[idx] = reference[idx];
                    }
                }
            }
            // Draw split line
            for y in 0..h {
                let idx = y * w + split_x;
                if idx < buf.len() {
                    buf[idx] = 0xFFFFFF;
                    if split_x + 1 < w && idx + 1 < buf.len() {
                        buf[idx + 1] = 0x000000;
                    }
                }
            }
        }
    }

    pub fn move_split(&mut self, delta: f32) {
        self.split_pos = (self.split_pos + delta).clamp(0.1, 0.9);
    }
}

// Helpers

fn fill_rect(buf: &mut [u32], buf_w: usize, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for row in y..y + h {
        for col in x..x + w {
            let idx = row * buf_w + col;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}

fn blend_color(base_color: u32, intensity: u32) -> u32 {
    let r = ((base_color >> 16) & 0xFF) * intensity / 255;
    let g = ((base_color >> 8) & 0xFF) * intensity / 255;
    let b = (base_color & 0xFF) * intensity / 255;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}
