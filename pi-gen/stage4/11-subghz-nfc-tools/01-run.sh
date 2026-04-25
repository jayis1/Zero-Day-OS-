#!/bin/bash -e
set -euo pipefail
# stage4/11-subghz-nfc-tools/01-run.sh — Install Sub-GHz and NFC tools

BIN="${ROOTFS_DIR}/usr/local/bin"

# ── Sub-GHz tools ──
for script in subghz-scan subghz-record subghz-replay; do
    if [ -f "${BASE_DIR}/../scripts/subghz/${script}" ]; then
        cp "${BASE_DIR}/../scripts/subghz/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    fi
done

# ── NFC tools ──
for script in nfc-read nfc-clone nfc-emulate; do
    if [ -f "${BASE_DIR}/../scripts/nfc/${script}" ]; then
        cp "${BASE_DIR}/../scripts/nfc/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    fi
done

# Install Sub-GHz + NFC packages
on_chroot << EOF
# RTL-SDR (software defined radio — USB dongle)
apt-get install -y --no-install-recommends rtl-433 2>/dev/null || true

# NFC core libraries
apt-get install -y --no-install-recommends \
    libnfc-bin \
    libnfc-dev \
    mfoc \
    pcscd \
    libpcsclite-dev 2>/dev/null || echo "[zeroday] Some NFC packages deferred"

# mfcuk may not be in Bookworm — try install, skip if not available
apt-get install -y --no-install-recommends mfcuk 2>/dev/null || echo "[zeroday] mfcuk not available in repos — build from source if needed"

# Python NFC library
pip3 install --break-system-packages nfcpy 2>/dev/null || echo "[zeroday] nfcpy pip install deferred"

# Python CC1101 library (SPI Sub-GHz)
pip3 install --break-system-packages cc1101 2>/dev/null || echo "[zeroday] cc1101 pip install deferred"

# RFCat (YardStick One support)
pip3 install --break-system-packages rfcat 2>/dev/null || echo "[zeroday] rfcat pip install deferred"
EOF

# Create NFC config
mkdir -p "${ROOTFS_DIR}/etc/nfc"
cat > "${ROOTFS_DIR}/etc/nfc/libnfc.conf" << 'NFCCONF'
# Allow automatic device scanning
allow_automatic_scan = 1
NFCCONF

# I2C device permissions for NFC
mkdir -p "${ROOTFS_DIR}/etc/udev/rules.d"
cat > "${ROOTFS_DIR}/etc/udev/rules.d/70-pn532.rules" << 'UDEVRULE'
# PN532 NFC module on I2C — ensure device node is accessible
KERNEL=="i2c-[0-9]*", MODE="0666"
# ACR122U USB NFC reader
SUBSYSTEM=="usb", ATTRS{idVendor}=="072f", ATTRS{idProduct}=="2200", MODE="0666"
UDEVRULE

# Ensure i2c_dev module loads
if ! grep -q "i2c_dev" "${ROOTFS_DIR}/etc/modules" 2>/dev/null; then
    echo "i2c_dev" >> "${ROOTFS_DIR}/etc/modules"
fi

echo "[zeroday] Sub-GHz and NFC tools installed."