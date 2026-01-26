#!/usr/bin/env bash
# Installer for codex-profiles
# Detects OS/arch, downloads binary from releases, verifies checksum from repo

set -euo pipefail

VERSION="${CODEX_PROFILES_VERSION:-0.1.0}"
REPO="midhunmonachan/codex-profiles"
INSTALL_DIR="${CODEX_PROFILES_INSTALL_DIR:-$HOME/.local/bin}"

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD='\033[1m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    RED='\033[0;31m'
    RESET='\033[0m'
else
    BOLD='' GREEN='' YELLOW='' RED='' RESET=''
fi

info() {
    printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$*" >&2
}

warn() {
    printf "${YELLOW}warning:${RESET} %s\n" "$*" >&2
}

error() {
    printf "${RED}error:${RESET} %s\n" "$*" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        error "need '$1' (command not found)"
    fi
}

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT) os="windows" ;;
        *)          error "unsupported OS: $(uname -s)" ;;
    esac

    local machine="$(uname -m)"
    case "$machine" in
        x86_64|amd64)       arch="x86_64" ;;
        aarch64|arm64)      arch="aarch64" ;;
        *)                  error "unsupported architecture: $machine" ;;
    esac

    case "$os-$arch" in
        linux-x86_64)       echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)      echo "aarch64-unknown-linux-gnu" ;;
        darwin-x86_64)      echo "x86_64-apple-darwin" ;;
        darwin-aarch64)     echo "aarch64-apple-darwin" ;;
        windows-x86_64)     echo "x86_64-pc-windows-msvc" ;;
        *)                  error "unsupported platform: $os-$arch" ;;
    esac
}

download_file() {
    local url="$1"
    local output="$2"
    
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 "$url" -o "$output" || return 1
    elif command -v wget > /dev/null 2>&1; then
        wget -q --https-only --secure-protocol=TLSv1_2 "$url" -O "$output" || return 1
    else
        error "need 'curl' or 'wget' to download"
    fi
}

verify_checksum() {
    local file="$1"
    local checksum_file="$2"
    
    local basename="$(basename "$file")"
    local expected actual
    
    expected="$(grep "release/$basename" "$checksum_file" | awk '{print $1}')"
    if [ -z "$expected" ]; then
        error "checksum not found for $basename in checksum file"
    fi
    
    if command -v sha256sum > /dev/null 2>&1; then
        actual="$(sha256sum "$file" | awk '{print $1}')"
    elif command -v shasum > /dev/null 2>&1; then
        actual="$(shasum -a 256 "$file" | awk '{print $1}')"
    else
        warn "sha256sum/shasum not found, skipping checksum verification"
        return 0
    fi
    
    if [ "$expected" != "$actual" ]; then
        error "checksum mismatch!\n  expected: $expected\n  actual:   $actual"
    fi
    
    info "Checksum verified ✓"
}

main() {
    need_cmd uname
    need_cmd mkdir
    need_cmd chmod
    need_cmd tar
    
    info "Installing codex-profiles v$VERSION"
    
    local target="$(detect_platform)"
    info "Detected platform: $target"
    
    local base_url="https://github.com/$REPO/releases/download/v$VERSION"
    local archive_name="codex-profiles-${target}.tar.gz"
    local archive_url="$base_url/$archive_name"
    
    local checksum_url="https://raw.githubusercontent.com/$REPO/main/checksums/v${VERSION}.txt"
    
    local tmpdir="$(mktemp -d)"
    trap "rm -rf '$tmpdir'" EXIT
    
    local archive_path="$tmpdir/$archive_name"
    local checksum_path="$tmpdir/checksums.txt"
    
    info "Downloading binary..."
    download_file "$archive_url" "$archive_path" || error "failed to download binary from $archive_url"
    
    info "Downloading checksums from repo..."
    if ! download_file "$checksum_url" "$checksum_path"; then
        warn "Could not download checksum file from repo"
        warn "Proceeding without verification (not recommended)"
    else
        verify_checksum "$archive_path" "$checksum_path"
    fi
    
    info "Extracting..."
    tar -xzf "$archive_path" -C "$tmpdir" || error "extraction failed"
    
    local binary_path
    if [ -f "$tmpdir/codex-profiles" ]; then
        binary_path="$tmpdir/codex-profiles"
    elif [ -f "$tmpdir/codex-profiles/codex-profiles" ]; then
        binary_path="$tmpdir/codex-profiles/codex-profiles"
    else
        error "binary not found in archive"
    fi
    
    info "Installing to $INSTALL_DIR/codex-profiles"
    mkdir -p "$INSTALL_DIR"
    
    if [ -f "$INSTALL_DIR/codex-profiles" ]; then
        local backup="$INSTALL_DIR/codex-profiles.backup.$(date +%s)"
        mv "$INSTALL_DIR/codex-profiles" "$backup"
        info "Backed up existing binary to $backup"
    fi
    
    cp "$binary_path" "$INSTALL_DIR/codex-profiles"
    chmod +x "$INSTALL_DIR/codex-profiles"
    
    if [ -x "$INSTALL_DIR/codex-profiles" ]; then
        local installed_version="$("$INSTALL_DIR/codex-profiles" --version 2>&1 | head -1)"
        info "Successfully installed: $installed_version"
    else
        error "installation failed: binary is not executable"
    fi
    
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        warn "$INSTALL_DIR is not in your PATH"
        warn "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    else
        info "Installation complete! Run: codex-profiles --help"
    fi
}

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Install codex-profiles by downloading the correct binary for your platform.

Options:
  -v, --version VERSION    Install specific version (default: $VERSION)
  -d, --dir DIR            Install to directory (default: $INSTALL_DIR)
  -h, --help               Show this help message

Environment variables:
  CODEX_PROFILES_VERSION          Override default version
  CODEX_PROFILES_INSTALL_DIR      Override default install directory
  NO_COLOR                        Disable colored output

Security:
  Checksums are downloaded from the git repository (separate from binaries)
  to protect against compromised release artifacts.

Examples:
  $0                              # Install latest (default: v$VERSION)
  $0 --version 0.2.0             # Install specific version
  $0 --dir /usr/local/bin        # Install to custom directory

EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--version)
            VERSION="${2#v}"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "unknown option: $1\nRun '$0 --help' for usage."
            ;;
    esac
done

main
