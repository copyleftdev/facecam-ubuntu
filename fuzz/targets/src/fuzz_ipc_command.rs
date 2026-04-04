/// Fuzz target: IPC command deserialization
///
/// The daemon accepts JSON-encoded DaemonCommand over a Unix socket.
/// Malformed JSON or unexpected fields should be handled gracefully.
use facecam_common::ipc::{DaemonCommand, DaemonResponse};
use std::io::{self, Read};

fn parse_command(data: &[u8]) -> Option<()> {
    // Try parsing as UTF-8 string first
    let text = std::str::from_utf8(data).ok()?;

    // Try parsing as DaemonCommand
    match serde_json::from_str::<DaemonCommand>(text) {
        Ok(cmd) => {
            // Exercise serialization roundtrip
            let serialized = serde_json::to_string(&cmd).ok()?;
            let _roundtrip: DaemonCommand = serde_json::from_str(&serialized).ok()?;

            // Exercise specific command variants
            match &cmd {
                DaemonCommand::ApplyProfile { name } => {
                    let _ = name.len();
                }
                DaemonCommand::SetControl { name, value } => {
                    let _ = name.len();
                    let _ = *value as i32;
                }
                DaemonCommand::GetControl { name } => {
                    let _ = name.len();
                }
                _ => {}
            }
        }
        Err(_) => {
            // Also try parsing as DaemonResponse (daemon might echo)
            let _ = serde_json::from_str::<DaemonResponse>(text);
        }
    }

    // Also try raw serde_json::Value to test edge cases
    let _ = serde_json::from_str::<serde_json::Value>(text);

    Some(())
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();
    let _ = parse_command(&input);
}
