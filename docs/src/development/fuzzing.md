# Fuzzing

The project includes AFL++ fuzz harnesses for all input parsing surfaces.

## Targets

| Target | Attack Surface | Input |
|--------|---------------|-------|
| `fuzz_v4l2_querycap` | VIDIOC_QUERYCAP response (104 bytes) | Kernel ioctl struct |
| `fuzz_v4l2_controls` | VIDIOC_QUERYCTRL response (68 bytes) | Kernel ioctl struct |
| `fuzz_ipc_command` | DaemonCommand JSON | Unix socket input |
| `fuzz_profile_parse` | Profile TOML | User-writable files |
| `fuzz_format_fourcc` | Pixel format + video mode | USB descriptor data |
| `fuzz_firmware_bcd` | Product ID + BCD version | USB descriptor data |

## Running

```bash
# Install AFL++ and cargo-afl
sudo apt-get install afl++
cargo install cargo-afl

# Build instrumented targets
cd fuzz/targets
cargo afl build --release

# Generate seed corpus
cd ../..
python3 fuzz/gen_corpus.py

# Run a single fuzzer
cargo afl fuzz -i fuzz/corpus/ipc_parse -o fuzz/findings/ipc_command \
    -- fuzz/targets/target/release/fuzz_ipc_command

# Run all fuzzers in parallel
for target in v4l2_querycap ipc_command profile_parse format_fourcc firmware_bcd; do
    corpus="fuzz/corpus/v4l2_parse"
    [ "$target" = "ipc_command" ] && corpus="fuzz/corpus/ipc_parse"
    [ "$target" = "profile_parse" ] && corpus="fuzz/corpus/profile_parse"
    [ "$target" = "format_fourcc" ] && corpus="fuzz/corpus/usb_descriptor"
    [ "$target" = "firmware_bcd" ] && corpus="fuzz/corpus/usb_descriptor"
    timeout 120 cargo afl fuzz -i "$corpus" -o "fuzz/findings/$target" \
        -- "fuzz/targets/target/release/fuzz_$target" &
done
wait
```

## Results

Initial fuzzing run (633,000+ executions across all targets):

| Target | Executions | Exec/sec | Crashes | Stability |
|--------|-----------|----------|---------|-----------|
| v4l2_querycap | 87,018 | 725/s | 0 | 100% |
| ipc_command | 309,552 | 2,581/s | 0 | 100% |
| profile_parse | 63,621 | 530/s | 0 | 97.4% |
| format_fourcc | 86,538 | 721/s | 0 | 100% |
| firmware_bcd | 86,529 | 721/s | 0 | 100% |

Zero crashes across all targets. Rust's type safety and bounds checking prevent the buffer overflow and integer overflow classes that AFL++ typically finds in C parsers.
