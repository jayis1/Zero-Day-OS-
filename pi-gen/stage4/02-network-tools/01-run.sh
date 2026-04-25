#!/bin/bash -e
set -euo pipefail
# stage4/02-network-tools/01-run.sh — Install network tools (Kali + scripts)

BIN="${ROOTFS_DIR}/usr/local/bin"

# Install network scripts
for script in net-discover net-quickscan net-vulnscan quick-c2 doh-proxy tunnel-mgr iot-scan; do
    if [ -f "${BASE_DIR}/../scripts/network/${script}" ]; then
        cp "${BASE_DIR}/../scripts/network/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    fi
done

# Responder — LLMNR/NBT-NS poisoner (Kali, best-effort)
on_chroot << EOF
apt-get -y -t kali-rolling install --no-install-recommends responder 2>/dev/null || echo "[zeroday] responder not available from Kali armhf — install manually if needed"
EOF

echo "[zeroday] Network tools installed."