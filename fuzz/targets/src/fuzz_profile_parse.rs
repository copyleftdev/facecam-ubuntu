/// Fuzz target: Profile TOML deserialization
///
/// Profiles are loaded from user-writable TOML files.
/// Malformed TOML, unexpected types, or huge values should be handled.
use facecam_common::profiles::Profile;
use std::io::{self, Read};

fn parse_profile(data: &[u8]) -> Option<()> {
    let text = std::str::from_utf8(data).ok()?;

    match toml::from_str::<Profile>(text) {
        Ok(profile) => {
            // Exercise all fields
            let _ = profile.name.len();
            let _ = profile.description.len();

            if let Some(ref vm) = profile.video_mode {
                // Check for insane dimensions that could cause OOM
                let pixels = (vm.width as u64).saturating_mul(vm.height as u64);
                if pixels > 100_000_000 {
                    return Some(()); // Absurd but shouldn't panic
                }
                let _ = vm.format.len();
            }

            // Exercise control map
            for (name, value) in &profile.controls {
                let _ = name.len();
                let _ = *value as i32; // Test truncation
            }

            // Roundtrip serialization
            let _ = toml::to_string_pretty(&profile);
        }
        Err(_) => {}
    }

    Some(())
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = parse_profile(&input);
}
