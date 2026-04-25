#!/bin/bash -e
set -euo pipefail
# stage4/12-meshtastic-tools/01-run.sh — Install Meshtastic mesh networking tools

BIN="${ROOTFS_DIR}/usr/local/bin"

# Install mesh-chat script
if [ -f "${BASE_DIR}/../scripts/mesh/mesh-chat" ]; then
    cp "${BASE_DIR}/../scripts/mesh/mesh-chat" "${BIN}/mesh-chat"
    chmod +x "${BIN}/mesh-chat"
    echo "[zeroday] Installed: mesh-chat"
fi

# Install Meshtastic Python CLI
on_chroot << EOF
pip3 install --break-system-packages meshtastic 2>/dev/null || echo "[zeroday] meshtastic pip install deferred to first boot"
apt-get install -y --no-install-recommends python3-serial 2>/dev/null || true
EOF

# Create Meshtastic config directory
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config/meshtastic"
cat > "${ROOTFS_DIR}/opt/cardputer/config/meshtastic/config.yaml" << 'MESHCONF'
# ZERO-DAY OS — Meshtastic Configuration
# Connected via Grove HY2.0-4P port (UART mode)

device:
  port: /dev/ttyUSB0
  baud: 115200

node:
  name: "zeroday"
  region: US
  channel: 1
  modem_preset: LongFast

bluetooth:
  enabled: false

wifi:
  enabled: false
MESHCONF

# Create udev rules for Meshtastic devices
mkdir -p "${ROOTFS_DIR}/etc/udev/rules.d"
cat > "${ROOTFS_DIR}/etc/udev/rules.d/70-meshtastic.rules" << 'UDEVRULE'
# Meshtastic USB-serial devices
SUBSYSTEM=="tty", ATTRS{idVendor}=="2886", ATTRS{idProduct}=="*002d", MODE="0666", SYMLINK+="meshtastic"
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666"
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="55d4", MODE="0666"
UDEVRULE

echo "[zeroday] Meshtastic tools installed."