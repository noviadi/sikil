#!/usr/bin/env bash
# Build script for Sikil release builds
# Usage: ./scripts/build.sh [target]
#   target - Optional Rust target triple (e.g., x86_64-unknown-linux-gnu)
#            Defaults to the host's target

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

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    log_error "cargo is not installed or not in PATH"
    exit 1
fi

# Parse arguments
TARGET="${1:-}"

log_info "Building Sikil..."

# Build command
BUILD_CMD=("cargo" "build" "--release")

if [[ -n "$TARGET" ]]; then
    log_info "Target: $TARGET"
    BUILD_CMD+=("--target" "$TARGET")

    # Check if target is installed
    if ! rustup target list --installed | grep -q "^$TARGET\$"; then
        log_warn "Target $TARGET is not installed. Installing..."
        rustup target add "$TARGET"
    fi
else
    # Get host target
    HOST_TARGET=$(rustc -vV | grep '^host:' | awk '{print $2}')
    log_info "Target: $HOST_TARGET (host)"
fi

# Run the build
log_info "Running: ${BUILD_CMD[*]}"
"${BUILD_CMD[@]}"

# Determine the binary path
if [[ -n "$TARGET" ]]; then
    BIN_DIR="target/$TARGET/release"
else
    BIN_DIR="target/release"
fi

BINARY="$BIN_DIR/sikil"

if [[ ! -f "$BINARY" ]]; then
    log_error "Binary not found at $BINARY"
    exit 1
fi

# Get binary size
BINARY_SIZE=$(du -h "$BINARY" | cut -f1)
log_info "Binary size: $BINARY_SIZE"

# Check if size is reasonable (<10MB)
BINARY_SIZE_BYTES=$(wc -c < "$BINARY")
MAX_SIZE=$((10 * 1024 * 1024)) # 10MB

if [[ $BINARY_SIZE_BYTES -gt $MAX_SIZE ]]; then
    log_warn "Binary size exceeds 10MB. Consider reviewing dependencies."
else
    log_info "Binary size is within acceptable range."
fi

log_info "Build complete: $BINARY"

# Test the binary
log_info "Running basic smoke test..."
if "$BINARY" --version &> /dev/null; then
    log_info "Smoke test passed."
else
    log_error "Smoke test failed. Binary may be corrupted."
    exit 1
fi
