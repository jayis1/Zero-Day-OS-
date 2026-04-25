#!/bin/bash -e
set -euo pipefail
# stage3/04-hardware-enable/01-run.sh — Enable Cardputer Zero hardware
# Enables I2C, SPI, camera, IR, and other peripherals
# Note: raspi-config is NOT available in pure Debian — use direct config instead

# ── Enable I2C, SPI, Camera via direct config (not raspi-config) ──
on_chroot << EOF
# Create I2C and SPI device nodes / permissions
mkdir -p /etc/modules-load.d
mkdir -p /etc/udev/rules.d

# Enable i2c and spi via systemd-modules-load
echo "i2c_dev" > /etc/modules-load.d/i2c.conf
echo "spi_bcm2835" >> /etc/modules-load.d/i2c.conf 2>/dev/null || true
echo "spidev" >> /etc/modules-load.d/i2c.conf 2>/dev/null || true

# Add the spidev overlay to config.txt if it exists (RPi boot)
if [ -f /boot/config.txt ]; then
    echo "dtparam=spi=on" >> /boot/config.txt 2>/dev/null || true
    echo "dtparam=i2c_arm=on" >> /boot/config.txt 2>/dev/null || true
fi
EOF

# ── Load required kernel modules ──
# Note: BCM2835-specific modules may not exist on all hardware.
# The build will be deployed on BCM2837-based Cardputer Zero,
# but we list alternatives and let the kernel skip unknown modules.
cat > "${ROOTFS_DIR}/etc/modules-load.d/zeroday.conf" << 'MODULES'
# ZERO-DAY OS kernel modules
i2c_dev
i2c_bcm2835
spi_bcm2835
spidev
brcmfmac
brcmutil
hci_uart
btbcm
btintel
btrtl
rfkill
lirc_dev
uinput
# Note: USB gadget modules (g_ether, g_serial, libcomposite) are NOT loaded
# statically because they conflict with each other and require configfs setup.
# They are loaded dynamically by usb-gadget-mode when the user activates them.
# Note: lirc_rpi does not exist in mainline kernels. Use lirc_dev + ir_gpio_tx/rx
# device tree overlays instead.
MODULES

# ── Configure RTC (RX8130CE) ──
cat > "${ROOTFS_DIR}/etc/modules-load.d/rtc.conf" << 'EOF'
rtc_rx8130
i2c_dev
EOF

# ── Set CPU governor via systemd service (not in chroot) ──
# Writing to /sys inside on_chroot affects the BUILD HOST, not the target.
# Instead, configure this as a first-boot service.
cat > "${ROOTFS_DIR}/etc/systemd/system/cpufreq-ondemand.service" << 'EOF'
[Unit]
Description=Set CPU governor to ondemand
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/sh -c 'echo ondemand > /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || true'

[Install]
WantedBy=multi-user.target
EOF

on_chroot << EOF
systemctl enable cpufreq-ondemand.service 2>/dev/null || true
systemctl enable i2c-dev.service 2>/dev/null || true
EOF

echo "[zeroday] Hardware modules configured."