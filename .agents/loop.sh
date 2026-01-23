#!/bin/bash
set -euo pipefail

# Usage: ./loop.sh [max_iterations]
# Examples:
#   ./loop.sh              # Default: 1 iteration (safe)
#   ./loop.sh 5            # Max 5 iterations
#   ./loop.sh 0            # Unlimited iterations (explicit)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_FILE="$SCRIPT_DIR/PROMPT.md"

# Parse arguments: default to 1 (defensive), 0 means unlimited
if [[ "${1:-}" =~ ^[0-9]+$ ]]; then
    MAX_ITERATIONS=$1
else
    MAX_ITERATIONS=1
fi

ITERATION=0
CURRENT_BRANCH=$(git branch --show-current)

# jq filter to extract streaming text from assistant messages and results
STREAM_TEXT='select(.type == "assistant").message.content[]? | select(.type == "text").text // empty | gsub("\n"; "\r\n") | . + "\r\n\n"'

# Validate git state
if [ -z "$CURRENT_BRANCH" ]; then
    echo "Error: Not on a branch (detached HEAD?)"
    exit 1
fi

# Verify prompt file exists
if [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: $PROMPT_FILE not found"
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Prompt: $PROMPT_FILE"
echo "Branch: $CURRENT_BRANCH"
if [ $MAX_ITERATIONS -eq 0 ]; then
    echo "Max:    unlimited (explicit)"
else
    echo "Max:    $MAX_ITERATIONS iteration(s)"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

while true; do
    if [ $MAX_ITERATIONS -gt 0 ] && [ $ITERATION -ge $MAX_ITERATIONS ]; then
        echo "Reached max iterations: $MAX_ITERATIONS"
        break
    fi

    ITERATION=$((ITERATION + 1))
    echo -e "\n======================== ITERATION $ITERATION ========================\n"

    # Temp file for raw output (cleanup on exit)
    tmpfile=$(mktemp)
    trap "rm -f $tmpfile" EXIT

    # Run Claude iteration with selected prompt
    # -p: Headless mode (non-interactive, reads from stdin)
    # --dangerously-skip-permissions: Auto-approve all tool calls
    # --output-format=stream-json: Structured output for logging/monitoring
    # --verbose: Detailed execution logging
    if ! cat "$PROMPT_FILE" | claude -p \
        --dangerously-skip-permissions \
        --output-format=stream-json \
        --verbose \
        | grep --line-buffered '^{' \
        | tee "$tmpfile" \
        | jq --unbuffered -rj "$STREAM_TEXT"; then
        echo "Error: Claude failed with exit code $?"
        exit 1
    fi

    # Push changes after each iteration (if nothing to push then just terminate)
    if ! git diff --quiet HEAD @{u} 2>/dev/null; then
        git push origin "$CURRENT_BRANCH" || {
            echo "Failed to push. Creating remote branch..."
            git push -u origin "$CURRENT_BRANCH"
        }
    else
        echo "No new commits to push after $ITERATION iteration"
        exit 0
    fi
done
