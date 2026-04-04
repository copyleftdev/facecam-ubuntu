#!/usr/bin/env bash
set -euo pipefail

# Facecam Ubuntu Runtime — Install Script
# Installs dependencies, builds Rust binaries, and configures the system.

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (sudo)."
        exit 1
    fi
}

install_system_deps() {
    info "Installing system dependencies..."
    apt-get update -qq
    apt-get install -y -qq \
        v4l2loopback-dkms \
        v4l2loopback-utils \
        v4l-utils \
        libusb-1.0-0-dev \
        pkg-config \
        build-essential
    info "System dependencies installed."
}

check_rust() {
    if ! command -v cargo &>/dev/null; then
        error "Rust toolchain not found. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    info "Rust toolchain found: $(rustc --version)"
}

build_binaries() {
    info "Building Rust binaries (release mode)..."
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"
    cargo build --release
    info "Build complete."
}

install_binaries() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local target_dir="$SCRIPT_DIR/target/release"

    info "Installing binaries to /usr/local/bin/..."
    install -m 755 "$target_dir/facecam-probe"   /usr/local/bin/facecam-probe
    install -m 755 "$target_dir/facecam-daemon"  /usr/local/bin/facecam-daemon
    install -m 755 "$target_dir/facecam-ctl"     /usr/local/bin/facecam-ctl
    install -m 755 "$target_dir/facecam-harness" /usr/local/bin/facecam-harness
    info "Binaries installed."
}

install_configs() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local config_dir="$SCRIPT_DIR/config"

    info "Installing udev rules..."
    install -m 644 "$config_dir/99-facecam.rules" /etc/udev/rules.d/99-facecam.rules
    udevadm control --reload-rules
    udevadm trigger

    info "Installing v4l2loopback configuration..."
    install -m 644 "$config_dir/v4l2loopback.conf" /etc/modprobe.d/v4l2loopback.conf
    install -m 644 "$config_dir/v4l2loopback-load.conf" /etc/modules-load.d/v4l2loopback.conf

    info "Installing systemd service..."
    install -m 644 "$config_dir/facecam-daemon.service" /etc/systemd/system/facecam-daemon.service
    systemctl daemon-reload

    info "System configuration installed."
}

load_v4l2loopback() {
    info "Loading v4l2loopback module..."
    if lsmod | grep -q v4l2loopback; then
        warn "v4l2loopback already loaded. Skipping."
    else
        modprobe v4l2loopback video_nr=10 card_label="Facecam Normalized" exclusive_caps=1 max_buffers=4 max_openers=10
        info "v4l2loopback loaded — /dev/video10 available."
    fi
}

create_user_config() {
    local user_home
    user_home=$(eval echo ~"${SUDO_USER:-$USER}")
    local config_dir="$user_home/.config/facecam"

    info "Creating user configuration..."
    sudo -u "${SUDO_USER:-$USER}" mkdir -p "$config_dir/profiles"

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [[ ! -f "$config_dir/daemon.toml" ]]; then
        sudo -u "${SUDO_USER:-$USER}" cp "$SCRIPT_DIR/config/daemon.toml" "$config_dir/daemon.toml"
    fi

    # Create default profiles
    sudo -u "${SUDO_USER:-$USER}" /usr/local/bin/facecam-ctl profile init 2>/dev/null || true
    info "User configuration created in $config_dir"
}

enable_service() {
    info "Enabling facecam-daemon service..."
    systemctl enable facecam-daemon.service
    info "Service enabled. It will start automatically when the Facecam is plugged in."
}

print_status() {
    echo ""
    info "=== Installation Complete ==="
    echo ""
    echo "  Binaries:"
    echo "    facecam-probe   — Detect and fingerprint the camera"
    echo "    facecam-daemon  — Normalization daemon (captures -> v4l2loopback)"
    echo "    facecam-ctl     — Control the daemon (profiles, controls, diagnostics)"
    echo "    facecam-harness — Automated compatibility testing"
    echo ""
    echo "  Quick start:"
    echo "    1. Plug in the Elgato Facecam (USB 3.0 port)"
    echo "    2. Run: facecam-probe detect"
    echo "    3. Start daemon: sudo systemctl start facecam-daemon"
    echo "    4. Use /dev/video10 (\"Facecam Normalized\") in applications"
    echo ""
    echo "  Useful commands:"
    echo "    facecam-probe detect        — Check if camera is detected"
    echo "    facecam-probe formats       — List supported video formats"
    echo "    facecam-probe quirks        — Show applicable device quirks"
    echo "    facecam-ctl status          — Show daemon status"
    echo "    facecam-ctl control list    — List camera controls"
    echo "    facecam-ctl profile list    — List available profiles"
    echo "    facecam-ctl profile apply streaming — Apply streaming profile"
    echo "    facecam-ctl diagnostics     — Export diagnostics bundle"
    echo "    facecam-harness full        — Run full compatibility test suite"
    echo ""
}

main() {
    echo "=== Facecam Ubuntu Runtime — Installer ==="
    echo ""

    check_root
    check_rust
    install_system_deps
    build_binaries
    install_binaries
    install_configs
    load_v4l2loopback
    create_user_config
    enable_service
    print_status
}

# Allow running individual steps
case "${1:-all}" in
    deps)     check_root; install_system_deps ;;
    build)    check_rust; build_binaries ;;
    install)  check_root; install_binaries; install_configs ;;
    config)   check_root; install_configs; create_user_config ;;
    all)      main ;;
    *)
        echo "Usage: $0 {all|deps|build|install|config}"
        exit 1
        ;;
esac
