# Building from Source

## Prerequisites

```bash
# Ubuntu 24.04+
sudo apt-get install -y \
    build-essential pkg-config \
    libusb-1.0-0-dev \
    libxkbcommon-dev libwayland-dev libx11-dev \
    v4l2loopback-dkms v4l-utils

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build

```bash
git clone https://github.com/facecam-ubuntu/facecam-ubuntu
cd facecam-ubuntu
cargo build --workspace --release
```

Binaries land in `target/release/`:
- `facecam-probe`
- `facecam-daemon`
- `facecam-ctl`
- `facecam-harness`
- `facecam-visual`

## Workspace Structure

```
Cargo.toml              # Workspace root
crates/
  facecam-common/       # Shared library (types, v4l2, USB, quirks, IPC)
  facecam-probe/        # Device probe CLI
  facecam-daemon/       # Normalization daemon
  facecam-ctl/          # Control CLI
  facecam-harness/      # Compatibility harness
  facecam-visual/       # Visual diagnostic tool
fuzz/
  targets/              # AFL++ fuzz harnesses (excluded from workspace)
  corpus/               # Seed inputs
config/                 # System config files (udev, systemd, modprobe)
docs/                   # This mdbook
research/               # Technical research memo
```

## Building the .deb

```bash
cargo build --workspace --release

PKG_DIR="facecam-ubuntu_0.1.0_amd64"
mkdir -p "$PKG_DIR"/{DEBIAN,usr/bin,etc/udev/rules.d,etc/modprobe.d,etc/modules-load.d,lib/systemd/system}
cp target/release/facecam-{probe,daemon,ctl,harness,visual} "$PKG_DIR/usr/bin/"
strip "$PKG_DIR/usr/bin/"facecam-*
cp config/99-facecam.rules "$PKG_DIR/etc/udev/rules.d/"
cp config/v4l2loopback.conf "$PKG_DIR/etc/modprobe.d/"
cp config/v4l2loopback-load.conf "$PKG_DIR/etc/modules-load.d/"
cp config/facecam-daemon.service "$PKG_DIR/lib/systemd/system/"

# Create DEBIAN/control, postinst, prerm (see CI workflow for full details)
dpkg-deb --build --root-owner-group "$PKG_DIR"
```

## Building the Docs

```bash
cd docs
mdbook build    # Output in docs/book/
mdbook serve    # Local preview at http://localhost:3000
```
