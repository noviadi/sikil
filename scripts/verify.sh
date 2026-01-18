#!/usr/bin/env bash
# Sikil - Single verification script
# Combines cargo test, clippy, and fmt check into one call
# Usage: ./scripts/verify.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS="${GREEN}✓${NC}"
FAIL="${RED}✗${NC}"

results=()

print_header() {
    echo ""
    echo -e "${YELLOW}▶ $1${NC}"
    echo "────────────────────────────────────"
}

run_check() {
    local name="$1"
    shift
    local cmd="$*"
    
    print_header "$name"
    echo "\$ $cmd"
    echo ""
    
    if eval "$cmd"; then
        results+=("$PASS $name")
        return 0
    else
        results+=("$FAIL $name")
        return 1
    fi
}

echo "╔════════════════════════════════════╗"
echo "║     Sikil Verification Suite       ║"
echo "╚════════════════════════════════════╝"

failed=0

run_check "cargo test" "cargo test" || failed=1
run_check "cargo clippy" "cargo clippy -- -D warnings" || failed=1
run_check "cargo fmt --check" "cargo fmt --check" || failed=1

echo ""
echo "════════════════════════════════════"
echo "SUMMARY"
echo "────────────────────────────────────"
for result in "${results[@]}"; do
    echo -e "  $result"
done
echo "════════════════════════════════════"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All checks passed${NC}"
    exit 0
else
    echo -e "${RED}Some checks failed${NC}"
    exit 1
fi
