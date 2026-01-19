#!/usr/bin/env bash
# Test script for Sikil
# Usage: ./scripts/test.sh [options]
#   --no-unit       Skip unit tests
#   --no-integration Skip integration tests
#   --release       Run tests in release mode (slower compile, faster tests)
#   --verbose       Enable verbose output

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly BLUE='\033[0;34m'
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

log_section() {
    echo ""
    echo -e "${BLUE}=== $* ===${NC}"
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
RUN_UNIT=true
RUN_INTEGRATION=true
RELEASE_MODE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-unit)
            RUN_UNIT=false
            shift
            ;;
        --no-integration)
            RUN_INTEGRATION=false
            shift
            ;;
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --no-unit        Skip unit tests"
            echo "  --no-integration Skip integration tests"
            echo "  --release        Run tests in release mode"
            echo "  --verbose        Enable verbose output"
            echo "  --help, -h       Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build cargo command
CARGO_TEST=("cargo" "test")

if [[ "$RELEASE_MODE" == true ]]; then
    CARGO_TEST+=("--release")
    log_info "Running tests in release mode"
fi

if [[ "$VERBOSE" == true ]]; then
    CARGO_TEST+=("--" "--nocapture")
fi

# Run unit tests (lib tests)
if [[ "$RUN_UNIT" == true ]]; then
    log_section "Running Unit Tests"
    if "${CARGO_TEST[@]}" --lib; then
        log_info "Unit tests passed"
    else
        log_error "Unit tests failed"
        exit 1
    fi
else
    log_warn "Skipping unit tests"
fi

# Run integration tests
if [[ "$RUN_INTEGRATION" == true ]]; then
    log_section "Running Integration Tests"
    if "${CARGO_TEST[@]}" --test '*'; then
        log_info "Integration tests passed"
    else
        log_error "Integration tests failed"
        exit 1
    fi
else
    log_warn "Skipping integration tests"
fi

# Run doctests
log_section "Running Doctests"
if "${CARGO_TEST[@]}" --doc; then
    log_info "Doctests passed"
else
    log_error "Doctests failed"
    exit 1
fi

# Check formatting (nightly only, skip if not available)
log_section "Checking Code Formatting"
if cargo fmt --version &> /dev/null; then
    if cargo fmt -- --check; then
        log_info "Code is properly formatted"
    else
        log_warn "Code formatting issues found. Run 'cargo fmt' to fix."
    fi
else
    log_warn "rustfmt not available, skipping format check"
fi

# Run clippy if available
log_section "Running Clippy"
if cargo clippy --version &> /dev/null; then
    if cargo clippy --all-targets --all-features -- -D warnings; then
        log_info "Clippy checks passed"
    else
        log_warn "Clippy found issues. Review warnings above."
    fi
else
    log_warn "clippy not available, skipping lint check"
fi

log_section "All Tests Passed"
log_info "Sikil is ready!"
