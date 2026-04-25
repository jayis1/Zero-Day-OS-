#!/bin/bash -e
set -euo pipefail
# stage3/01-kernel-dtb/01-run.sh — Custom kernel + device tree overlays
# Compiles DTS overlays and installs kernel config

# Compile all device tree overlays from the project
OVERLAY_SRC="${BASE_DIR}/../overlays"
OVERLAY_DST="${ROOTFS_DIR}/boot/overlays"

mkdir -p "${OVERLAY_DST}"

if [ -d "${OVERLAY_SRC}" ]; then
    for dts in "${OVERLAY_SRC}"/*.dts; do
        if [ -f "$dts" ]; then
            name=$(basename "$dts" .dts)
            # Strip -overlay suffix to match dtoverlay= naming convention
            # e.g., bmi270-overlay.dts -> bmi270.dtbo (dtoverlay=bmi270)
            overlay_name="${name%-overlay}"
            dtbo="${OVERLAY_DST}/${overlay_name}.dtbo"
            echo "[zeroday] Compiling overlay: ${name} -> ${overlay_name}.dtbo"
            dtc -@ -I dts -O dtb -o "${dtbo}" "$dts" 2>/dev/null || {
                echo "[zeroday] WARNING: Failed to compile ${name}.dts — device tree compiler not available in build env"
                echo "[zeroday] Overlays will need to be compiled on the target device"
            }
        fi
    done
else
    echo "[zeroday] WARNING: No device tree overlay sources found at ${OVERLAY_SRC}"
fi

# Install kernel config fragment
KERNEL_CFG="${BASE_DIR}/../kernel/zeroday-fragment.config"
if [ -f "${KERNEL_CFG}" ]; then
    mkdir -p "${ROOTFS_DIR}/boot/config-overlays"
    cp "${KERNEL_CFG}" "${ROOTFS_DIR}/boot/config-overlays/zeroday.conf"
    echo "[zeroday] Installed kernel config fragment to /boot/config-overlays/zeroday.conf"
fi

# Configure /boot/config.txt for Cardputer Zero
cat > "${ROOTFS_DIR}/boot/config.txt" << 'BOOTCFG'
# ZERO-DAY OS — M5Stack Cardputer Zero Boot Configuration

# Disable HDMI (1.9" LCD is primary display)
hdmi_force_hotplug=0
hdmi_drive=2

# GPU memory — minimal (TUI only, no desktop compositing)
gpu_mem=16

# Framebuffer
max_framebuffers=1
framebuffer_width=320
framebuffer_height=240

# Disable camera LED (stealth)
disable_camera_led=1

# Boot delay (0 for fast boot)
boot_delay=0

# Disable splash screen (we have our own boot animation)
disable_splash=1

# ── Device Tree Overlays ──
# Cardputer Zero hardware
dtoverlay=st7789v3
dtoverlay=cardputer-kbd
dtoverlay=es8389
dtoverlay=imx219
dtoverlay=bmi270
dtoverlay=rx8130ce
dtoverlay=bq27220
dtoverlay=ir-trx

# I2C and SPI
dtparam=i2c1=on
dtparam=i2c_arm=on
dtparam=spi=on

# Disable audio (we use ES8389, not BCM PWM audio)
dtparam=audio=off
BOOTCFG

echo "[zeroday] Boot configuration written."