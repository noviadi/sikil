#!/usr/bin/env bash
# Sikil - Unified task completion script
# Runs verify → complete-task → commit in one atomic operation
# Usage: ./scripts/finish-task.sh <TASK_ID> <AGENT> "<DESCRIPTION>" --subtask "S01:<evidence>" ...

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    echo "Usage: $0 <TASK_ID> <AGENT> \"<DESCRIPTION>\" --subtask \"S01:<evidence>\" ..."
    echo ""
    echo "Arguments:"
    echo "  TASK_ID       Task identifier (e.g., M2-E01-T05)"
    echo "  AGENT         Agent name (e.g., claude, amp-agent)"
    echo "  DESCRIPTION   Brief description for commit subject"
    echo ""
    echo "Options:"
    echo "  --subtask \"Sxx:<evidence>\"   Subtask evidence (can be repeated)"
    echo "  --notes \"<notes>\"            Optional notes for LOG.md (defaults to DESCRIPTION)"
    echo "  --force                       Skip focus mismatch check"
    echo "  --no-verify                   Skip verification (use with caution)"
    echo ""
    echo "Example:"
    echo "  $0 M2-E01-T05 claude \"implement cache integration\" \\"
    echo "    --subtask \"S01:Added CacheService struct\" \\"
    echo "    --subtask \"S02:Unit tests in cache_test.rs\""
    exit 1
}

# Parse arguments
FORCE=""
NO_VERIFY=false
NOTES=""
SUBTASKS=()
POSITIONAL=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --subtask)
            if [[ -z "${2:-}" ]]; then
                echo -e "${RED}Error: --subtask requires an argument${NC}"
                exit 1
            fi
            SUBTASKS+=("$2")
            shift 2
            ;;
        --notes)
            if [[ -z "${2:-}" ]]; then
                echo -e "${RED}Error: --notes requires an argument${NC}"
                exit 1
            fi
            NOTES="$2"
            shift 2
            ;;
        --force)
            FORCE="--force"
            shift
            ;;
        --no-verify)
            NO_VERIFY=true
            echo -e "${YELLOW}⚠ WARNING: Skipping verification (--no-verify)${NC}"
            shift
            ;;
        -h|--help)
            usage
            ;;
        -*)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            ;;
        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done

if [ ${#POSITIONAL[@]} -ne 3 ]; then
    echo -e "${RED}Error: Expected 3 positional arguments (TASK_ID, AGENT, DESCRIPTION)${NC}"
    usage
fi

TASK_ID="${POSITIONAL[0]}"
AGENT="${POSITIONAL[1]}"
DESCRIPTION="${POSITIONAL[2]}"

# Default notes to description if not provided
if [ -z "$NOTES" ]; then
    NOTES="$DESCRIPTION"
fi

# Validate TASK_ID format
if ! echo "$TASK_ID" | grep -qE '^M[0-9]+-E[0-9]+-T[0-9]+$'; then
    echo -e "${RED}Error: Invalid TASK_ID format '$TASK_ID'${NC}"
    echo "Expected format: M<n>-E<n>-T<n> (e.g., M2-E01-T05)"
    exit 1
fi

# Validate at least one subtask provided
if [ ${#SUBTASKS[@]} -eq 0 ]; then
    echo -e "${RED}Error: At least one --subtask is required${NC}"
    echo "Example: --subtask \"S01:Added implementation\""
    exit 1
fi

# Validate subtask format
for subtask in "${SUBTASKS[@]}"; do
    if ! echo "$subtask" | grep -qE '^S[0-9]+:.+'; then
        echo -e "${RED}Error: Invalid subtask format '$subtask'${NC}"
        echo "Expected format: S<nn>:<evidence> (e.g., S01:Added cache module)"
        exit 1
    fi
done

echo "╔════════════════════════════════════╗"
echo "║     Sikil Task Completion          ║"
echo "╚════════════════════════════════════╝"
echo ""
echo "Task: $TASK_ID"
echo "Agent: $AGENT"
echo "Description: $DESCRIPTION"
echo ""

# Step 1: Run verification (unless --no-verify)
if [ "$NO_VERIFY" = false ]; then
    echo -e "${YELLOW}▶ Step 1: Running verification...${NC}"
    echo ""
    
    # Capture verification output
    VERIFY_OUTPUT=$(mktemp)
    if ! "$SCRIPT_DIR/verify.sh" 2>&1 | tee "$VERIFY_OUTPUT"; then
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}Verification FAILED. Task completion aborted.${NC}"
        echo -e "${RED}Fix the issues above and run again.${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
        rm -f "$VERIFY_OUTPUT"
        exit 1
    fi
    
    # Extract results from verification output
    TEST_RESULT="passed"
    CLIPPY_RESULT="passed"
    FMT_RESULT="passed"
    
    rm -f "$VERIFY_OUTPUT"
    echo ""
else
    echo -e "${YELLOW}▶ Step 1: Skipping verification (--no-verify)${NC}"
    TEST_RESULT="skipped"
    CLIPPY_RESULT="skipped"
    FMT_RESULT="skipped"
    echo ""
fi

# Step 2: Run complete-task.sh
echo -e "${YELLOW}▶ Step 2: Updating STATE.yaml and LOG.md...${NC}"
echo ""

if ! "$SCRIPT_DIR/complete-task.sh" "$TASK_ID" "$AGENT" "$NOTES" $FORCE; then
    echo ""
    echo -e "${RED}Error: complete-task.sh failed${NC}"
    exit 1
fi

# Step 3: Generate commit message and commit
echo ""
echo -e "${YELLOW}▶ Step 3: Creating commit...${NC}"
echo ""

# Generate commit message
COMMIT_MSG=$(mktemp)
cat > "$COMMIT_MSG" << EOF
$TASK_ID: $DESCRIPTION

Verification:
- cargo test: $TEST_RESULT
- cargo clippy -- -D warnings: $CLIPPY_RESULT
- cargo fmt --check: $FMT_RESULT

Subtasks:
EOF

for subtask in "${SUBTASKS[@]}"; do
    echo "- $subtask" >> "$COMMIT_MSG"
done

# Add co-author
echo "" >> "$COMMIT_MSG"
echo "Co-Authored-By: $AGENT <noreply@agent.local>" >> "$COMMIT_MSG"

# Validate commit message
if ! "$SCRIPT_DIR/validate-commit-msg.sh" "$COMMIT_MSG"; then
    echo ""
    echo -e "${RED}Error: Generated commit message failed validation${NC}"
    cat "$COMMIT_MSG"
    rm -f "$COMMIT_MSG"
    exit 1
fi

# Stage all changes
git add -A

# Commit
if ! git commit -F "$COMMIT_MSG"; then
    echo ""
    echo -e "${RED}Error: git commit failed${NC}"
    echo "Commit message was saved to: $COMMIT_MSG"
    exit 1
fi

rm -f "$COMMIT_MSG"

# Get commit hash
COMMIT_HASH=$(git rev-parse HEAD)
COMMIT_SHORT=$(git rev-parse --short HEAD)

echo ""
echo "════════════════════════════════════════════════════════════"
echo -e "${GREEN}✓ Task $TASK_ID completed successfully${NC}"
echo "────────────────────────────────────────────────────────────"
echo "  Commit: $COMMIT_SHORT ($COMMIT_HASH)"
echo "════════════════════════════════════════════════════════════"
