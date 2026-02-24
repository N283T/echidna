#!/bin/sh
# Echidna installer script
# Usage: curl -sSfL https://raw.githubusercontent.com/N283T/echidna/master/install.sh | sh

set -e

REPO="N283T/echidna"
BINARY_NAME="echidna"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() {
    printf "${CYAN}info:${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}success:${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warning:${NC} %s\n" "$1"
}

error() {
    printf "${RED}error:${NC} %s\n" "$1" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)       error "Unsupported operating system: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)            error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Build download URL
build_download_url() {
    local version="$1"
    local os="$2"
    local arch="$3"

    local target=""
    case "${os}-${arch}" in
        linux-x86_64)   target="x86_64-unknown-linux-gnu" ;;
        macos-x86_64)   target="x86_64-apple-darwin" ;;
        macos-aarch64)  target="aarch64-apple-darwin" ;;
        windows-x86_64) target="x86_64-pc-windows-msvc" ;;
        *)              error "Unsupported platform: ${os}-${arch}" ;;
    esac

    local ext="tar.gz"
    if [ "$os" = "windows" ]; then
        ext="zip"
    fi

    echo "https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}.${ext}"
}

# Determine install directory
get_install_dir() {
    if [ -w "/usr/local/bin" ]; then
        echo "/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        echo "$HOME/.local/bin"
    else
        mkdir -p "$HOME/.local/bin"
        echo "$HOME/.local/bin"
    fi
}

# Main installation
main() {
    info "Detecting platform..."
    local os=$(detect_os)
    local arch=$(detect_arch)
    info "Platform: ${os}-${arch}"

    info "Fetching latest version..."
    local version=$(get_latest_version)
    if [ -z "$version" ]; then
        error "Failed to fetch latest version"
    fi
    info "Latest version: ${version}"

    local url=$(build_download_url "$version" "$os" "$arch")
    info "Downloading from: ${url}"

    local tmpdir=$(mktemp -d)
    trap "rm -rf '$tmpdir'" EXIT

    local archive="${tmpdir}/echidna.tar.gz"
    if [ "$os" = "windows" ]; then
        archive="${tmpdir}/echidna.zip"
    fi

    # Download
    if command -v curl >/dev/null 2>&1; then
        curl -sSfL "$url" -o "$archive" || error "Download failed"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$archive" || error "Download failed"
    else
        error "Neither curl nor wget found"
    fi

    # Extract
    info "Extracting..."
    if [ "$os" = "windows" ]; then
        unzip -q "$archive" -d "$tmpdir"
    else
        tar -xzf "$archive" -C "$tmpdir"
    fi

    # Install
    local install_dir=$(get_install_dir)
    local binary="${tmpdir}/${BINARY_NAME}"
    if [ "$os" = "windows" ]; then
        binary="${binary}.exe"
    fi

    if [ ! -f "$binary" ]; then
        # Try to find binary in extracted directory
        binary=$(find "$tmpdir" -name "${BINARY_NAME}*" -type f | head -1)
    fi

    if [ ! -f "$binary" ]; then
        error "Binary not found in archive"
    fi

    info "Installing to ${install_dir}..."
    mv "$binary" "${install_dir}/${BINARY_NAME}"
    chmod +x "${install_dir}/${BINARY_NAME}"

    success "Echidna ${version} installed successfully!"

    # Check if install_dir is in PATH
    case ":$PATH:" in
        *":${install_dir}:"*) ;;
        *)
            warn "Add ${install_dir} to your PATH:"
            echo ""
            echo "  export PATH=\"${install_dir}:\$PATH\""
            echo ""
            ;;
    esac

    echo ""
    info "Run 'echidna --help' to get started"
}

main "$@"
