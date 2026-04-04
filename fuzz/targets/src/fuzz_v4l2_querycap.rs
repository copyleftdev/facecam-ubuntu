/// Fuzz target: V4L2 QUERYCAP response parsing
///
/// The querycap response is a 104-byte struct from the kernel.
/// We parse driver name, card name, bus info, version, and capability flags.
/// A malicious/buggy device could return garbage here.
use std::io::{self, Read};

fn parse_querycap(data: &[u8]) -> Option<()> {
    if data.len() < 104 {
        return None;
    }

    // Same parsing logic as v4l2::query_capabilities
    let driver = read_fixed_string(&data[0..16]);
    let card = read_fixed_string(&data[16..48]);
    let bus_info = read_fixed_string(&data[48..80]);
    let version = u32::from_ne_bytes(data[80..84].try_into().ok()?);
    let capabilities = u32::from_ne_bytes(data[84..88].try_into().ok()?);
    let device_caps = u32::from_ne_bytes(data[88..92].try_into().ok()?);

    let effective_caps = if capabilities & 0x80000000 != 0 {
        device_caps
    } else {
        capabilities
    };

    // Exercise the parsed values — ensure no panics
    let _ = format!(
        "{}.{}.{}",
        (version >> 16) & 0xFF,
        (version >> 8) & 0xFF,
        version & 0xFF
    );
    let has_capture = effective_caps & 0x00000001 != 0;
    let has_output = effective_caps & 0x00000002 != 0;
    let has_streaming = effective_caps & 0x04000000 != 0;
    let has_readwrite = effective_caps & 0x01000000 != 0;

    // Ensure string parsing doesn't produce invalid UTF-8 panics
    let _ = driver.len();
    let _ = card.len();
    let _ = bus_info.len();
    let _ = (has_capture, has_output, has_streaming, has_readwrite);

    Some(())
}

fn read_fixed_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).to_string()
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = parse_querycap(&input);
}
