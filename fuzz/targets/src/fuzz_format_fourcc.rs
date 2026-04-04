/// Fuzz target: Pixel format fourcc parsing and video mode calculations
///
/// Tests PixelFormat::from_fourcc, bandwidth calculation, and display formatting.
/// Ensures no panics on arbitrary 4-byte fourcc codes.
use facecam_common::formats::{PixelFormat, VideoMode};
use std::io::{self, Read};

fn fuzz_formats(data: &[u8]) -> Option<()> {
    if data.len() < 16 {
        return None;
    }

    // Parse fourcc from first 4 bytes
    let fourcc = u32::from_le_bytes(data[0..4].try_into().ok()?);
    let format = PixelFormat::from_fourcc(fourcc);

    // Exercise all methods
    let _ = format.to_fourcc();
    let _ = format.fourcc_str();
    let _ = format.bytes_per_pixel();
    let _ = format.is_reliable_on_facecam();
    let _ = format!("{}", format);

    // Parse video mode from remaining bytes
    let width = u32::from_le_bytes(data[4..8].try_into().ok()?);
    let height = u32::from_le_bytes(data[8..12].try_into().ok()?);
    let fps_num = u16::from_le_bytes(data[12..14].try_into().ok()?) as u32;
    let fps_den = u16::from_le_bytes(data[14..16].try_into().ok()?) as u32;

    let mode = VideoMode {
        format,
        width,
        height,
        fps_numerator: fps_num,
        fps_denominator: fps_den,
    };

    // Exercise calculations — must not panic on zero/overflow
    let _ = mode.fps();
    let _ = mode.bandwidth_bytes_per_sec();
    let _ = format!("{}", mode);

    Some(())
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = fuzz_formats(&input);
}
