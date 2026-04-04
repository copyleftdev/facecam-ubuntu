# Installation

## From .deb Package (Recommended)

Download the latest `.deb` from the [Releases page](https://github.com/facecam-ubuntu/facecam-ubuntu/releases) and install:

```bash
sudo dpkg -i facecam-ubuntu_0.1.0_amd64.deb
sudo apt-get install -f
```

This installs:
- 5 binaries to `/usr/bin/`
- udev rules to `/etc/udev/rules.d/99-facecam.rules`
- v4l2loopback config to `/etc/modprobe.d/v4l2loopback.conf`
- Module autoload to `/etc/modules-load.d/v4l2loopback-load.conf`
- systemd service to `/lib/systemd/system/facecam-daemon.service`

Dependencies (`v4l2loopback-dkms`, `v4l-utils`, `libusb-1.0-0`) are pulled automatically.

## From Source

### Prerequisites

```bash
# System packages
sudo apt-get install -y \
    v4l2loopback-dkms v4l2loopback-utils v4l-utils \
    libusb-1.0-0-dev pkg-config build-essential

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Install

```bash
git clone https://github.com/facecam-ubuntu/facecam-ubuntu
cd facecam-ubuntu
sudo ./install.sh
```

Or build manually:

```bash
cargo build --workspace --release
sudo cp target/release/facecam-{probe,daemon,ctl,harness,visual} /usr/local/bin/
sudo cp config/99-facecam.rules /etc/udev/rules.d/
sudo cp config/v4l2loopback.conf /etc/modprobe.d/
sudo cp config/facecam-daemon.service /etc/systemd/system/
sudo udevadm control --reload-rules
sudo systemctl daemon-reload
```

### Load v4l2loopback

```bash
sudo modprobe v4l2loopback video_nr=10 card_label="Facecam Normalized" exclusive_caps=1
```

Verify:
```bash
ls /dev/video10  # Should exist
```

## Uninstall

```bash
sudo dpkg -r facecam-ubuntu
```

Or manually remove binaries from `/usr/local/bin/` and config files.
