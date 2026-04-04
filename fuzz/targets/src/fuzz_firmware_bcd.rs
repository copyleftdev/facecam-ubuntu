/// Fuzz target: Firmware version BCD parsing and product ID mapping
///
/// Tests FirmwareVersion::from_bcd, ElgatoProduct::from_pid, and
/// the quirk registry matching logic with arbitrary inputs.
use facecam_common::device::{ElgatoProduct, FirmwareVersion};
use facecam_common::quirks;
use std::io::{self, Read};

fn fuzz_device(data: &[u8]) -> Option<()> {
    if data.len() < 4 {
        return None;
    }

    let pid = u16::from_le_bytes(data[0..2].try_into().ok()?);
    let bcd = u16::from_le_bytes(data[2..4].try_into().ok()?);

    let product = ElgatoProduct::from_pid(pid);
    let firmware = FirmwareVersion::from_bcd(bcd);

    // Exercise all methods
    let _ = product.pid();
    let _ = product.name();
    let _ = product.is_facecam_original();
    let _ = product.is_usb2_fallback();
    let _ = format!("{}", product);

    let _ = firmware.has_mjpeg();
    let _ = firmware.has_transfer_mode_selection();
    let _ = format!("{}", firmware);

    // Exercise quirk matching — this allocates and leaks (by design)
    // so only do it for a subset to avoid OOM
    if data.len() >= 5 && data[4] & 0x01 == 0 {
        let quirks = quirks::applicable_quirks(product, firmware);
        for q in &quirks {
            let _ = q.id.len();
            let _ = q.summary.len();
            let _ = format!("{:?}", q.severity);
        }
    }

    Some(())
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = fuzz_device(&input);
}
