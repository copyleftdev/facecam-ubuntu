# Contributing

## Getting Started

1. Fork the repository
2. Clone and build: `cargo build --workspace`
3. Run tests: `cargo test --workspace`
4. Make changes on a feature branch

## Code Style

- `cargo fmt` before committing
- `cargo clippy -- -D warnings` must pass
- No new warnings in any crate

## Adding a New Quirk

Observed a new device behavior? Add it to the registry:

1. Document the symptoms, trigger, and firmware version in a GitHub issue
2. Add the quirk to `crates/facecam-common/src/quirks.rs`
3. Add a test to the harness if the behavior is testable
4. Update `docs/src/architecture/quirks.md`

## Adding a New Control

If a firmware update exposes new V4L2 controls:

1. Verify with `v4l2-ctl --list-ctrls` and document the CID, range, default
2. Add to `v4l2::control_name_to_id()` mapping
3. Update `docs/src/device/controls.md`
4. Update default profiles if the control has useful presets

## Pull Request Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo build --workspace --release` succeeds
- [ ] New quirks have traceable observations
- [ ] Docs updated for user-facing changes
- [ ] Commit messages describe *why*, not just *what*

## Architecture Notes

- **facecam-common** is the shared library — put types, parsing, and protocol logic here
- **Binary crates** should be thin wrappers around common functionality
- **V4L2 ioctls** use raw byte arrays with verified struct offsets (see `v4l2.rs` comments)
- **No C dependencies** in the core library except libc and libusb
