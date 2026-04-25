#!/bin/bash -e
set -euo pipefail
# stage4/03-bluetooth-tools/01-run.sh — Install Bluetooth tools and MITM framework

# bettercap — Swiss-army MITM framework (Kali armhf, best-effort)
on_chroot << EOF
apt-get -y -t kali-rolling install --no-install-recommends bettercap 2>/dev/null || echo "[zeroday] bettercap not available from Kali armhf — install manually if needed"
EOF

echo "[zeroday] Bluetooth tools installed."