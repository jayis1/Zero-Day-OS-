#!/bin/bash -e
set -euo pipefail
# stage3/11-ratatui-tui/01-run.sh
# Install zeroday-tui ratatui dashboards for ZERO-DAY OS
# Builds: sys-tui, net-tui, loot-tui, scan-tui
# Also installs ratatui-upgraded zeroday-fm (file explorer)

BIN="${ROOTFS_DIR}/usr/local/bin"
TUI_DIR="${PROJECT_ROOT}/tui/target/aarch64-unknown-linux-gnu/release"
FM_BIN="${PROJECT_ROOT}/explorer/target/aarch64-unknown-linux-gnu/release/zeroday-fm"

install -m 755 -d "${BIN}"

echo "[zeroday] Installing ratatui TUI dashboards..."

# Install zeroday-tui binaries (pre-built by cross-compile step)
for bin in sys-tui net-tui loot-tui scan-tui trail-tui wifi-tui bt-tui; do
    if [ -f "${TUI_DIR}/${bin}" ]; then
        install -m 755 "${TUI_DIR}/${bin}" "${BIN}/${bin}"
        SIZE=$(du -h "${TUI_DIR}/${bin}" | cut -f1)
        echo "[zeroday] Installed: ${bin} (${SIZE})"
    else
        echo "[zeroday] WARNING: ${bin} not found — run: cd tui && make cross-build"
    fi
done

# Install ratatui-upgraded file explorer (replaces crossterm-only version)
if [ -f "${FM_BIN}" ]; then
    install -m 755 "${FM_BIN}" "${BIN}/zeroday-fm"
    SIZE=$(du -h "${FM_BIN}" | cut -f1)
    echo "[zeroday] Installed: zeroday-fm (ratatui, ${SIZE})"
else
    echo "[zeroday] WARNING: zeroday-fm not found — run: cd explorer && make cross-build"
fi

echo "[zeroday] Ratatui TUI dashboards installed."
echo "[zeroday]   sys-tui   — system monitor (CPU/RAM/battery/temp/processes/sparklines)"
echo "[zeroday]   net-tui   — network dashboard (live nmap XML reader, host/port tables)"
echo "[zeroday]   loot-tui  — loot browser (category tabs, file preview, cred highlights)"
echo "[zeroday]   trail-tui — navigation overlay (breadcrumb trail, exit guidance, Overwatch threats)"
echo "[zeroday]   wifi-tui  — WiFi dashboard (live scan, wardrive history, handshake/crack, attacks)"
echo "[zeroday]   bt-tui    — Bluetooth dashboard (BT+BLE scan, GATT enum, device history, attacks)"
