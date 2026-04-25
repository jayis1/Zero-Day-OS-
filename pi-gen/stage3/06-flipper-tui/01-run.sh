#!/bin/bash -e
# stage3/06-flipper-tui/01-run.sh — Install the Flipper-style TUI application

BIN="${ROOTFS_DIR}/usr/local/bin"
TUI_SRC="${BASE_DIR}/../tui"

# Install cyber_launcher.py
if [ -f "${TUI_SRC}/cyber_launcher.py" ]; then
    cp "${TUI_SRC}/cyber_launcher.py" "${BIN}/cyber_launcher"
    chmod +x "${BIN}/cyber_launcher"
    echo "[zeroday] Installed: cyber_launcher"
else
    echo "[zeroday] WARNING: cyber_launcher.py not found"
    # Create a minimal fallback
    cat > "${BIN}/cyber_launcher" << 'FALLBACK'
#!/usr/bin/env python3
"""ZERO-DAY OS — Flipper TUI (fallback)"""
import os, sys
print("ZERO-DAY OS TUI")
print("===============")
print("Categories: WIFI DONGLE NET BT IR CAM PAYLD RADIO SHELL SYS OPEN")
print("Run individual commands from shell, or install full TUI.")
FALLBACK
    chmod +x "${BIN}/cyber_launcher"
fi

# Install Textual CSS if it exists
if [ -f "${TUI_SRC}/cyber_launcher.tcss" ]; then
    mkdir -p "${ROOTFS_DIR}/opt/cardputer/tui"
    cp "${TUI_SRC}/cyber_launcher.tcss" "${ROOTFS_DIR}/opt/cardputer/tui/"
fi

# Ensure Textual is installed
on_chroot << EOF
pip3 install textual 2>/dev/null || echo "[zeroday] Textual pip install deferred to first boot"
EOF

echo "[zeroday] Flipper TUI installed."