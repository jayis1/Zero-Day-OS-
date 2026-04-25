#!/bin/bash -e
set -euo pipefail
# stage4/10-wifi-dongle/01-run.sh — Install RTL8821CU dongle driver and udev rules
# Note: linux-headers-rpi is a Raspbian package, not available in Debian.
# We install generic kernel headers and defer DKMS build to first boot.

BIN="${ROOTFS_DIR}/usr/local/bin"

# Install the dongle-setup script
if [ -f "${BASE_DIR}/../scripts/dongle/dongle-setup" ]; then
    cp "${BASE_DIR}/../scripts/dongle/dongle-setup" "${BIN}/dongle-setup"
    chmod +x "${BIN}/dongle-setup"
fi

# Create udev rule for consistent WLAN naming
mkdir -p "${ROOTFS_DIR}/etc/udev/rules.d"
cat > "${ROOTFS_DIR}/etc/udev/rules.d/70-rtl8821cu.rules" << 'UDEVRULE'
# RTL8821CU USB WiFi dongle — always assign to wlan1
SUBSYSTEM=="net", ACTION=="add", DRIVERS=="?*", ATTRS{idVendor}=="0bda", ATTRS{idProduct}=="c811|1a2b", NAME="wlan1"
UDEVRULE

# Add module to /etc/modules — but note: 8821cu/rtl8821cu are out-of-tree
# drivers that need to be built via DKMS on first boot.
# Only add them if the module exists; skip if not built yet.
# The dongle-setup script will handle building the driver on first boot.
if ! grep -q "# rtl8821cu" "${ROOTFS_DIR}/etc/modules" 2>/dev/null; then
    cat >> "${ROOTFS_DIR}/etc/modules" << 'MODULES'
# rtl8821cu — requires DKMS build on first boot (dongle-setup)
# 8821cu
# rtl8821cu
MODULES
fi

# Install build dependencies for DKMS driver build on first boot
# Use Debian kernel headers instead of RPi-specific ones
on_chroot << EOF
apt-get install -y --no-install-recommends \
    linux-headers-generic \
    dkms \
    bc \
    git \
    build-essential \
    libelf-dev 2>/dev/null || echo "[zeroday] DKMS build deps install deferred"
EOF

echo "[zeroday] RTL8821CU dongle support configured."