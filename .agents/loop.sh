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

tmpfile=""
cleanup() { [[ -n "${tmpfile:-}" ]] && rm -f -- "$tmpfile"; }
trap cleanup EXIT

while true; do
    if [ $MAX_ITERATIONS -gt 0 ] && [ $ITERATION -ge $MAX_ITERATIONS ]; then
        echo "Reached max iterations: $MAX_ITERATIONS"
        break
    fi

    ITERATION=$((ITERATION + 1))
    echo -e "\n======================== ITERATION $ITERATION ========================\n"

    # Capture HEAD before agent run to detect if agent made commits
    before_head=$(git rev-parse HEAD)

    # Temp file for raw output (cleaned up at end of iteration or on exit)
    tmpfile=$(mktemp)

    # Run Claude iteration with selected prompt
    # -p: Headless mode (non-interactive, reads from stdin)
    # --dangerously-skip-permissions: Auto-approve all tool calls
    # --output-format=stream-json: Structured output for logging/monitoring
    # --verbose: Detailed execution logging
    set +e
    cat "$PROMPT_FILE" | claude -p \
        --dangerously-skip-permissions \
        --output-format=stream-json \
        --verbose \
        | grep --line-buffered '^{' \
        | tee "$tmpfile" \
        | jq --unbuffered -rj "$STREAM_TEXT"
    pipe_status=("${PIPESTATUS[@]}")
    set -e

    # Check pipeline stages for failures
    for s in "${pipe_status[@]}"; do
        if [[ "$s" -ne 0 ]]; then
            echo "Error: Agent pipeline failed with status: ${pipe_status[*]}"
            exit 1
        fi
    done

    # Verify branch didn't change during agent run
    after_branch=$(git symbolic-ref --quiet --short HEAD) || {
        echo "Error: Detached HEAD after agent run"
        exit 1
    }
    if [[ "$after_branch" != "$CURRENT_BRANCH" ]]; then
        echo "Error: Branch changed during agent run: $CURRENT_BRANCH -> $after_branch"
        exit 1
    fi

    # Compare HEAD to detect if agent made commits
    after_head=$(git rev-parse HEAD)

    if [[ "$after_head" == "$before_head" ]]; then
        # No commit created: verify working tree is clean (agent shouldn't leave uncommitted changes)
        if ! git diff --quiet || ! git diff --cached --quiet; then
            echo "Error: No commit created but working tree is dirty (agent violated workflow)"
            git status --porcelain
            exit 1
        fi
        echo "No new commits after $ITERATION iteration(s) — agent found no remaining work"
        break
    fi

    # Agent created commit(s): push to remote
    git fetch origin "$CURRENT_BRANCH" 2>/dev/null || true
    git push -u origin "$CURRENT_BRANCH"

    # Clean up temp file before next iteration
    cleanup
done
