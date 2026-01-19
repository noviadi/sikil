#!/usr/bin/env bash
# Sikil - Commit message validator
# Validates commit message format for task completion
# Usage: ./scripts/validate-commit-msg.sh <commit-msg-file>

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ $# -lt 1 ]; then
    echo -e "${RED}Usage: $0 <commit-msg-file>${NC}"
    exit 1
fi

MSG_FILE="$1"

if [ ! -f "$MSG_FILE" ]; then
    echo -e "${RED}Error: Commit message file not found: $MSG_FILE${NC}"
    exit 1
fi

MSG=$(cat "$MSG_FILE")

errors=()

# Rule 1: Header line matches task ID format
HEADER=$(echo "$MSG" | head -n1)
if ! echo "$HEADER" | grep -qE '^M[0-9]+-E[0-9]+-T[0-9]+: .+'; then
    errors+=("Header must match format '<TASK_ID>: <description>' (e.g., 'M2-E01-T05: implement feature')")
fi

# Rule 2: Blank line after header
SECOND_LINE=$(echo "$MSG" | sed -n '2p')
if [ -n "$SECOND_LINE" ]; then
    errors+=("Second line must be blank")
fi

# Rule 3: Verification section exists
if ! echo "$MSG" | grep -q '^Verification:'; then
    errors+=("Missing 'Verification:' section")
else
    # Check verification bullets
    if ! echo "$MSG" | grep -q '^- cargo test:'; then
        errors+=("Missing '- cargo test: <result>' in Verification section")
    fi
    if ! echo "$MSG" | grep -q '^- cargo clippy .*-D warnings:'; then
        errors+=("Missing '- cargo clippy -- -D warnings: <result>' in Verification section")
    fi
    if ! echo "$MSG" | grep -q '^- cargo fmt .*check:'; then
        errors+=("Missing '- cargo fmt --check: <result>' in Verification section")
    fi
fi

# Rule 4: Subtasks section exists with at least one entry
if ! echo "$MSG" | grep -q '^Subtasks:'; then
    errors+=("Missing 'Subtasks:' section")
else
    # Check for at least one subtask entry
    if ! echo "$MSG" | grep -qE '^- S[0-9]+: .+'; then
        errors+=("Subtasks section must have at least one entry (e.g., '- S01: <evidence>')")
    fi
fi

# Report results
if [ ${#errors[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ Commit message format is valid${NC}"
    exit 0
else
    echo -e "${RED}✗ Commit message format is invalid${NC}"
    echo ""
    echo "Errors:"
    for error in "${errors[@]}"; do
        echo -e "  ${RED}•${NC} $error"
    done
    echo ""
    echo -e "${YELLOW}Expected format:${NC}"
    echo "  <TASK_ID>: <description>"
    echo ""
    echo "  Verification:"
    echo "  - cargo test: <result>"
    echo "  - cargo clippy -- -D warnings: <result>"
    echo "  - cargo fmt --check: <result>"
    echo ""
    echo "  Subtasks:"
    echo "  - S01: <evidence>"
    echo "  - S02: <evidence>"
    exit 1
fi
