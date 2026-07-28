#!/bin/bash
# jesh install script
# Usage: curl -fsSL https://jesh.jefferson.app.br/install.sh | bash

set -euo pipefail

JESH_REPO="jefferson-it/jesh"
INSTALL_DIR="${HOME}/.local/bin"
CARGO_BIN="${HOME}/.cargo/bin"

detect_pkg_mgr() {
    if command -v apt >/dev/null 2>&1; then
        echo "apt"
    elif command -v dnf >/dev/null 2>&1; then
        echo "dnf"
    elif command -v yum >/dev/null 2>&1; then
        echo "yum"
    elif command -v pacman >/dev/null 2>&1; then
        echo "pacman"
    elif command -v yay >/dev/null 2>&1; then
        echo "yay"
    elif command -v paru >/dev/null 2>&1; then
        echo "paru"
    elif command -v zypper >/dev/null 2>&1; then
        echo "zypper"
    elif command -v apk >/dev/null 2>&1; then
        echo "apk"
    elif command -v brew >/dev/null 2>&1; then
        echo "brew"
    else
        echo "unknown"
    fi
}

install_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
}

install_via_cargo() {
    echo "Installing jesh via cargo..."
    install_rust
    cargo install jesh
    echo "Installed to ~/.cargo/bin/jesh"
}

install_from_source() {
    echo "Building jesh from source..."
    install_rust
    local tmpdir=$(mktemp -d)
    git clone "https://github.com/${JESH_REPO}.git" "$tmpdir/jesh"
    cd "$tmpdir/jesh"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/jesh "$INSTALL_DIR/"
    echo "Installed to $INSTALL_DIR/jesh"
    cd - >/dev/null
    rm -rf "$tmpdir"
}

install_debian() {
    echo "Installing dependencies for Debian/Ubuntu..."
    sudo apt update
    sudo apt install -y git curl build-essential pkg-config libssl-dev
    install_from_source
}

install_fedora() {
    echo "Installing dependencies for Fedora/RHEL..."
    sudo dnf install -y git curl gcc openssl-devel pkgconfig make
    install_from_source
}

install_arch() {
    echo "Installing dependencies for Arch/Manjaro..."
    sudo pacman -S --needed --noconfirm git curl base-devel openssl pkgconf
    install_from_source
}

install_opensuse() {
    echo "Installing dependencies for openSUSE..."
    sudo zypper install -y git curl gcc openssl-devel pkgconf make
    install_from_source
}

install_alpine() {
    echo "Installing dependencies for Alpine..."
    sudo apk add git curl build-base openssl-dev pkgconf
    install_from_source
}

install_macos() {
    if ! command -v brew >/dev/null 2>&1; then
        echo "Homebrew not found. Installing via cargo..."
        install_via_cargo
        return
    fi
    echo "Installing dependencies via Homebrew..."
    brew install git curl openssl pkg-config
    install_from_source
}

main() {
    echo "🐚 jesh installer"
    echo "=================="

    local pkg_mgr=$(detect_pkg_mgr)
    echo "Detected package manager: $pkg_mgr"

    case "$pkg_mgr" in
        apt)
            install_debian
            ;;
        dnf|yum)
            install_fedora
            ;;
        pacman|yay|paru)
            install_arch
            ;;
        zypper)
            install_opensuse
            ;;
        apk)
            install_alpine
            ;;
        brew)
            install_macos
            ;;
        *)
            echo "Unknown package manager. Falling back to cargo install..."
            install_via_cargo
            ;;
    esac

    echo ""
    echo "✅ jesh installed!"
    echo ""
    echo "Add to PATH if needed:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo ""
    echo "Then run: jesh"
}

main "$@"