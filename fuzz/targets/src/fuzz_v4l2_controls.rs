/// Fuzz target: V4L2 QUERYCTRL response parsing
///
/// The queryctrl response is a 68-byte struct. We parse control ID, type,
/// name, min/max/step/default, and flags. Bogus values could cause panics
/// in downstream logic (e.g., menu enumeration with huge ranges).
use std::io::{self, Read};

fn parse_queryctrl(data: &[u8]) -> Option<()> {
    if data.len() < 68 {
        return None;
    }

    let ctrl_id = u32::from_ne_bytes(data[0..4].try_into().ok()?);
    let ctrl_type_raw = u32::from_ne_bytes(data[4..8].try_into().ok()?);
    let name = read_fixed_string(&data[8..40]);
    let minimum = i32::from_ne_bytes(data[40..44].try_into().ok()?);
    let maximum = i32::from_ne_bytes(data[44..48].try_into().ok()?);
    let step = i32::from_ne_bytes(data[48..52].try_into().ok()?);
    let default = i32::from_ne_bytes(data[52..56].try_into().ok()?);
    let flags = u32::from_ne_bytes(data[56..60].try_into().ok()?);

    // Exercise control type mapping
    let ctrl_type = match ctrl_type_raw {
        1 => "integer",
        2 => "boolean",
        3 => "menu",
        4 => "button",
        5 => "integer64",
        6 => "ctrl_class",
        7 => "string",
        8 => "bitmask",
        9 => "integer_menu",
        _ => "unknown",
    };

    // Validate that range calculations don't overflow
    if step > 0 && maximum >= minimum {
        let range = (maximum as i64) - (minimum as i64);
        let _num_steps = range / (step as i64);
    }

    // Check for menu bounds sanity — prevent OOM from huge ranges
    if ctrl_type == "menu" && maximum >= minimum {
        let menu_size = (maximum as u64).saturating_sub(minimum as u64);
        if menu_size > 1000 {
            // Would be insane for a real device, but shouldn't panic
            return Some(());
        }
    }

    let _ = format!(
        "{}(0x{:08x}): type={} range=[{},{}] step={} default={} flags=0x{:x}",
        name, ctrl_id, ctrl_type, minimum, maximum, step, default, flags
    );

    Some(())
}

fn read_fixed_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).to_string()
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = parse_queryctrl(&input);
}
