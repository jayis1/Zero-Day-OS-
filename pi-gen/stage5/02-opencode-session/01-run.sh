#!/bin/bash -e
# stage5/02-opencode-session/01-run.sh — Install opencode-session wrapper

BIN="${ROOTFS_DIR}/usr/local/bin"

if [ -f "${BASE_DIR}/../scripts/system/opencode-session" ]; then
    cp "${BASE_DIR}/../scripts/system/opencode-session" "${BIN}/opencode-session"
    chmod +x "${BIN}/opencode-session"
    echo "[zeroday] Installed: opencode-session"
else
    cat > "${BIN}/opencode-session" << 'OPENCODE'
#!/bin/bash
# /usr/local/bin/opencode-session
# tmux split-screen IDE: OpenCode + live console

SESSION="opencode"
WORKSPACE="/opt/cardputer/workspace"

mkdir -p "$WORKSPACE"

# If session exists, attach; otherwise create
if tmux has-session -t "$SESSION" 2>/dev/null; then
    tmux attach -t "$SESSION"
    exit 0
fi

# Determine working directory
DIR="${1:-$WORKSPACE}"
FILE="$2"

cd "$DIR" || exit 1

# Create new tmux session
tmux new-session -d -s "$SESSION" -c "$DIR"

# Split horizontally: 70% top, 30% bottom
tmux split-window -v -p 30 -t "$SESSION" -c "$DIR"

# Top pane: OpenCode (or the file specified)
if [ -n "$FILE" ] && [ -f "$DIR/$FILE" ]; then
    tmux send-keys -t "$SESSION:0.0" "opencode $DIR/$FILE" Enter
else
    tmux send-keys -t "$SESSION:0.0" "opencode" Enter
fi

# Bottom pane: bash
tmux send-keys -t "$SESSION:0.1" "" Enter

# Select top pane
tmux select-pane -t "$SESSION:0.0"

# Attach
tmux attach -t "$SESSION"
OPENCODE
    chmod +x "${BIN}/opencode-session"
fi

echo "[zeroday] OpenCode session wrapper installed."