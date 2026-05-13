#!/bin/bash -e
set -euo pipefail
# stage3/01-kernel-dtb/01-run.sh — Custom kernel + device tree overlays
# Compiles DTS overlays and installs kernel config for Cardputer Zero

# ── Compile device tree overlays ──
OVERLAY_SRC="${BASE_DIR}/../overlays"
OVERLAY_DST="${ROOTFS_DIR}/boot/overlays"

mkdir -p "${OVERLAY_DST}"

if [ -d "${OVERLAY_SRC}" ]; then
    for dts in "${OVERLAY_SRC}"/*.dts; do
        if [ -f "$dts" ]; then
            name=$(basename "$dts" .dts)
            # Strip -overlay suffix to match dtoverlay= naming convention
            # e.g., cardputerzero-overlay.dts -> cardputerzero.dtbo (dtoverlay=cardputerzero)
            overlay_name="${name%-overlay}"
            dtbo="${OVERLAY_DST}/${overlay_name}.dtbo"
            echo "[zeroday] Compiling overlay: ${name} -> ${overlay_name}.dtbo"
            dtc -@ -I dts -O dtb -o "${dtbo}" "$dts" 2>/dev/null || {
                echo "[zeroday] WARNING: Failed to compile ${name}.dts — will try on target device"
                # Copy the .dts source so it can be compiled on target if dtc is missing in build env
                cp "$dts" "${OVERLAY_DST}/${name}.dts"
            }
        fi
    done
else
    echo "[zeroday] WARNING: No device tree overlay sources found at ${OVERLAY_SRC}"
fi

# ── Install kernel config fragment ──
KERNEL_CFG="${BASE_DIR}/../kernel/zeroday-fragment.config"
if [ -f "${KERNEL_CFG}" ]; then
    mkdir -p "${ROOTFS_DIR}/boot/config-overlays"
    cp "${KERNEL_CFG}" "${ROOTFS_DIR}/boot/config-overlays/zeroday.conf"
    echo "[zeroday] Installed kernel config fragment to /boot/config-overlays/zeroday.conf"
fi

# ── Download official CM0 firmware (kernel, DTBs, bootloader) ──
# The official M5Stack build uses pre-built firmware from dianjixz/cm0-firmware
# We download and install these alongside our custom overlays
FIRMWARE_DIR="${ROOTFS_DIR}/boot"
echo "[zeroday] Downloading official CM0 firmware..."

# Download the official firmware release if available
OFFICIAL_FIRMWARE_URL="https://github.com/dianjixz/cm0-firmware/archive/refs/heads/main.tar.gz"
FIRMWARE_TMP="$(mktemp -d)"

if command -v curl &>/dev/null; then
    echo "[zeroday] Fetching firmware from ${OFFICIAL_FIRMWARE_URL}..."
    if curl -sL "${OFFICIAL_FIRMWARE_URL}" -o "${FIRMWARE_TMP}/cm0-firmware.tar.gz" 2>/dev/null; then
        mkdir -p "${FIRMWARE_TMP}/extract"
        tar -xzf "${FIRMWARE_TMP}/cm0-firmware.tar.gz" -C "${FIRMWARE_TMP}/extract" 2>/dev/null || {
            echo "[zeroday] WARNING: Could not extract firmware archive — using stock RPi kernel"
        }
        # Copy kernel, DTBs, and overlays from official firmware
        FIRMWARE_EXTRACT_DIR=$(find "${FIRMWARE_TMP}/extract" -maxdepth 1 -type d -name "cm0-firmware*" | head -1)
        if [ -n "${FIRMWARE_EXTRACT_DIR}" ] && [ -d "${FIRMWARE_EXTRACT_DIR}" ]; then
            # Copy boot files (kernel, DTBs, start.elf, etc.)
            if [ -d "${FIRMWARE_EXTRACT_DIR}/boot" ]; then
                for f in "${FIRMWARE_EXTRACT_DIR}/boot/"*; do
                    [ -f "$f" ] && cp "$f" "${FIRMWARE_DIR}/" 2>/dev/null || true
                done
                echo "[zeroday] Installed official CM0 boot files"
            fi
            # Copy DTB overlays from official firmware
            if [ -d "${FIRMWARE_EXTRACT_DIR}/boot/overlays" ]; then
                for f in "${FIRMWARE_EXTRACT_DIR}/boot/overlays/"*.dtbo; do
                    [ -f "$f" ] && cp "$f" "${OVERLAY_DST}/" 2>/dev/null || true
                done
                echo "[zeroday] Installed official CM0 DTB overlays"
            fi
            # Copy kernel modules
            if [ -d "${FIRMWARE_EXTRACT_DIR}/modules" ]; then
                KVER=$(ls "${FIRMWARE_EXTRACT_DIR}/modules/" | head -1)
                if [ -n "$KVER" ]; then
                    mkdir -p "${ROOTFS_DIR}/lib/modules/${KVER}"
                    cp -r "${FIRMWARE_EXTRACT_DIR}/modules/${KVER}/"* "${ROOTFS_DIR}/lib/modules/${KVER}/" 2>/dev/null || true
                    echo "[zeroday] Installed kernel modules for ${KVER}"
                fi
            fi
        fi
    else
        echo "[zeroday] WARNING: Could not download official CM0 firmware — using stock RPi kernel"
    fi
else
    echo "[zeroday] WARNING: curl not available — skipping firmware download"
fi

rm -rf "${FIRMWARE_TMP}"

# ── Configure /boot/config.txt for Cardputer Zero ──
cat > "${ROOTFS_DIR}/boot/config.txt" << 'BOOTCFG'
# ZERO-DAY OS — M5Stack Cardputer Zero Boot Configuration
# Hardware: RP3A0 SoC (Pi Zero 2W die), 512MB LPDDR2, aarch64

# Disable HDMI (1.9" LCD is primary display)
hdmi_force_hotplug=0
hdmi_drive=2

# GPU memory — minimal (TUI/LVGL only, no desktop compositing)
gpu_mem=16

# Framebuffer — 320x170 for ST7789V LCD
max_framebuffers=1
framebuffer_width=320
framebuffer_height=170

# Disable camera LED (stealth)
disable_camera_led=1

# Boot delay (0 for fast boot)
boot_delay=0

# Disable splash screen (we have our own boot animation)
disable_splash=1

# ── Device Tree Overlays ──
# Main CardputerZero overlay (ST7789V LCD, TCA8418 keyboard, ES8390 audio, etc.)
dtoverlay=cardputerzero

# Additional overlays (from official m5stack-linux-dtoverlays + our custom)
dtoverlay=camera-gpio16-high
dtoverlay=spk-gpio24-high

# I2C and SPI (enabled by cardputerzero overlay but explicit here for clarity)
dtparam=i2c1=on
dtparam=i2c_arm=on
dtparam=spi=on

# Disable BCM PWM audio (we use ES8390 via I2S)
dtparam=audio=off
BOOTCFG

echo "[zeroday] Boot configuration written."