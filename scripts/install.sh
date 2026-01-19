#!/usr/bin/env bash
# Install script for Sikil
# Usage: ./scripts/install.sh [options]
#   --local         Install to ~/.cargo/bin/ using cargo install
#   --system        Install to /usr/local/bin/ (requires sudo)
#   --prefix PATH   Install to custom prefix
#   --uninstall     Uninstall Sikil

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Parse arguments
INSTALL_MODE="local"
PREFIX=""
UNINSTALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            INSTALL_MODE="local"
            shift
            ;;
        --system)
            INSTALL_MODE="system"
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --local         Install to ~/.cargo/bin/ (default)"
            echo "  --system        Install to /usr/local/bin/ (requires sudo)"
            echo "  --prefix PATH   Install to custom prefix"
            echo "  --uninstall     Uninstall Sikil"
            echo "  --help, -h      Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Handle uninstall
if [[ "$UNINSTALL" == true ]]; then
    log_info "Uninstalling Sikil..."

    # Check various locations
    LOCATIONS=(
        "$HOME/.cargo/bin/sikil"
        "/usr/local/bin/sikil"
        "$HOME/.local/bin/sikil"
    )

    # Also check custom prefix
    if [[ -n "$PREFIX" ]]; then
        LOCATIONS+=("$PREFIX/bin/sikil")
    fi

    REMOVED=0
    for loc in "${LOCATIONS[@]}"; do
        if [[ -f "$loc" ]]; then
            log_info "Removing: $loc"
            rm -f "$loc"
            REMOVED=1
        fi
    done

    if [[ $REMOVED -eq 1 ]]; then
        log_info "Sikil uninstalled successfully"
    else
        log_warn "No Sikil installation found"
    fi
    exit 0
fi

# Handle install
log_info "Installing Sikil..."

if [[ "$INSTALL_MODE" == "local" ]]; then
    log_info "Installing to ~/.cargo/bin/ using cargo install..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo is not installed or not in PATH"
        exit 1
    fi

    # Build and install locally
    cargo install --path .

    # Verify installation
    if command -v sikil &> /dev/null; then
        VERSION=$(sikil --version)
        log_info "Sikil installed successfully: $VERSION"
        log_info "Binary location: $(which sikil)"
    else
        log_error "Installation appears to have failed. 'sikil' command not found."
        log_error "Make sure ~/.cargo/bin is in your PATH."
        exit 1
    fi

elif [[ "$INSTALL_MODE" == "system" ]]; then
    # Determine install directory
    if [[ -n "$PREFIX" ]]; then
        BINDIR="$PREFIX/bin"
    else
        BINDIR="/usr/local/bin"
    fi

    log_info "Installing to $BINDIR..."

    # Check if we need sudo
    if [[ ! -w "$BINDIR" ]]; then
        if command -v sudo &> /dev/null; then
            log_info "Using sudo for installation"
            SUDO="sudo"
        else
            log_error "Cannot write to $BINDIR and sudo is not available"
            exit 1
        fi
    else
        SUDO=""
    fi

    # Build the release binary first
    log_info "Building release binary..."
    if ! cargo build --release; then
        log_error "Build failed"
        exit 1
    fi

    # Install the binary
    log_info "Copying binary to $BINDIR/sikil..."
    $SUDO cp target/release/sikil "$BINDIR/sikil"
    $SUDO chmod +x "$BINDIR/sikil"

    # Verify installation
    if command -v sikil &> /dev/null; then
        VERSION=$(sikil --version)
        log_info "Sikil installed successfully: $VERSION"
        log_info "Binary location: $(which sikil)"
    else
        log_warn "Installation completed, but 'sikil' command not found in PATH."
        log_warn "Make sure $BINDIR is in your PATH."
    fi
fi

log_info "Installation complete!"
log_info "Run 'sikil --help' to get started."
