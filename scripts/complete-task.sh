#!/usr/bin/env bash
# Sikil - Atomic task completion script
# Updates STATE.yaml and LOG.md in one operation
# Usage: ./scripts/complete-task.sh <TASK_ID> <AGENT> "<NOTES>" [--force]

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

STATE_FILE="docs/plan/STATE.yaml"
LOG_FILE="docs/plan/LOG.md"

# Portable sed -i (macOS BSD sed vs GNU sed)
sedi() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "$@"
    else
        sed -i "$@"
    fi
}

usage() {
    echo "Usage: $0 <TASK_ID> <AGENT> \"<NOTES>\" [--force]"
    echo ""
    echo "Arguments:"
    echo "  TASK_ID   Task identifier (e.g., M2-E01-T05)"
    echo "  AGENT     Agent name (e.g., claude, amp-agent)"
    echo "  NOTES     Brief summary of work done"
    echo "  --force   Skip focus mismatch check"
    echo ""
    echo "Example:"
    echo "  $0 M2-E01-T05 claude \"Implemented cache integration with 8 tests\""
    exit 1
}

# Parse arguments
FORCE=false
POSITIONAL=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE=true
            shift
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
    usage
fi

TASK_ID="${POSITIONAL[0]}"
AGENT="${POSITIONAL[1]}"
NOTES="${POSITIONAL[2]}"

# Validate files exist
if [ ! -f "$STATE_FILE" ]; then
    echo -e "${RED}Error: $STATE_FILE not found${NC}"
    exit 1
fi

if [ ! -f "$LOG_FILE" ]; then
    echo -e "${RED}Error: $LOG_FILE not found${NC}"
    exit 1
fi

# Get current timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
DATE_HEADER=$(date -u +"%Y-%m-%d")

# Validate current focus matches task (safety check)
CURRENT_FOCUS=$(grep -A1 "^focus:" "$STATE_FILE" | grep "current_task:" | sed 's/.*current_task: *//' | tr -d '"' | tr -d ' ')

if [ "$CURRENT_FOCUS" != "$TASK_ID" ] && [ "$CURRENT_FOCUS" != "null" ]; then
    if [ "$FORCE" = true ]; then
        echo -e "${YELLOW}Warning: Current focus is '$CURRENT_FOCUS', not '$TASK_ID' (--force used)${NC}"
    else
        echo -e "${RED}Error: Current focus is '$CURRENT_FOCUS', not '$TASK_ID'${NC}"
        echo "Use --force to override this check"
        exit 1
    fi
fi

echo -e "${YELLOW}▶ Completing task: $TASK_ID${NC}"
echo ""

# --- Update STATE.yaml ---
echo "Updating $STATE_FILE..."

# Update updated_at
sedi "s/^updated_at:.*/updated_at: \"$TIMESTAMP\"/" "$STATE_FILE"

# Clear focus
sedi '/^focus:/,/^[^ ]/{
  s/current_task:.*/current_task: null/
  s/by:.*/by: null/
}' "$STATE_FILE"

# Add task entry if not exists, or update if exists
if grep -q "\"$TASK_ID\":" "$STATE_FILE"; then
    # Update existing entry
    sedi "s/\"$TASK_ID\":.*$/\"$TASK_ID\": { status: done, at: \"$TIMESTAMP\" }/" "$STATE_FILE"
else
    # Add new entry before the last task or at end of items section
    # Find the last line of items section and append
    sedi "/^items:/a\\  \"$TASK_ID\": { status: done, at: \"$TIMESTAMP\" }" "$STATE_FILE"
fi

echo -e "  ${GREEN}✓${NC} STATE.yaml updated"

# --- Update LOG.md ---
echo "Updating $LOG_FILE..."

# Check if today's date header exists
if ! grep -q "^## $DATE_HEADER" "$LOG_FILE"; then
    # Add date header
    echo "" >> "$LOG_FILE"
    echo "---" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
    echo "## $DATE_HEADER" >> "$LOG_FILE"
fi

# Append log entry
cat >> "$LOG_FILE" << EOF

### $TASK_ID — $AGENT — done
- **Completed**: $TIMESTAMP
- **Notes**: $NOTES
EOF

echo -e "  ${GREEN}✓${NC} LOG.md updated"

# --- Summary ---
echo ""
echo "════════════════════════════════════"
echo -e "${GREEN}Task $TASK_ID marked complete${NC}"
echo "────────────────────────────────────"
echo "  Timestamp: $TIMESTAMP"
echo "  Agent: $AGENT"
echo "  Notes: $NOTES"
echo "════════════════════════════════════"
echo ""
echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
echo -e "${RED}REQUIRED: Commit all changes now.${NC}"
echo -e "${RED}Task is NOT complete until committed.${NC}"
echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "  git add -A && git commit"
echo ""
echo "Include: implementation files + $STATE_FILE + $LOG_FILE"
echo "Final response MUST include the commit hash."
