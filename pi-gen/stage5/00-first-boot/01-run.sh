#!/bin/bash -e
set -euo pipefail
# stage5/00-first-boot/01-run.sh — First boot wizard (runs once on first boot)

# Create first-boot systemd service
cat > "${ROOTFS_DIR}/etc/systemd/system/first-boot.service" << 'EOF'
[Unit]
Description=ZERO-DAY OS First Boot Setup
After=multi-user.target zeroday-boot.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/first-boot
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

# Create first-boot script
BIN="${ROOTFS_DIR}/usr/local/bin"
if [ -f "${BASE_DIR}/../scripts/system/first-boot" ]; then
    cp "${BASE_DIR}/../scripts/system/first-boot" "${BIN}/first-boot"
    chmod +x "${BIN}/first-boot"
else
    cat > "${BIN}/first-boot" << 'FIRSTBOOT'
#!/bin/bash
# /usr/local/bin/first-boot — Runs once on first boot
set -e

echo "╔══════════════════════════════════════════════════╗"
echo "║  ZERO-DAY OS  ·  FIRST BOOT SETUP               ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# Expand filesystem to fill SD card
echo "[*] Expanding filesystem..."
ROOT_DEV=$(findmnt -n -o SOURCE /)
if [ -n "${ROOT_DEV}" ]; then
    ROOT_PART="${ROOT_DEV##*p}"
    ROOT_DISK="${ROOT_DEV%p${ROOT_PART}}"
    # Grow partition to fill disk
    parted -s "${ROOT_DISK}" resizepart "${ROOT_PART}" 100% 2>/dev/null || true
    resize2fs "${ROOT_DEV}" 2>/dev/null || true
fi

# Switch resolv.conf to systemd-resolved stub
# (Cannot do this during build because chroot has no running systemd-resolved)
echo "[*] Configuring DNS via systemd-resolved..."
ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf 2>/dev/null || true

# Generate SSH host keys
echo "[*] Generating SSH host keys..."
ssh-keygen -A 2>/dev/null || true

# Write release info
cat > /etc/zeroday-release << RELEASE
ZERO-DAY OS v0.1-pre
Build: PLACEHOLDER
Kernel: $(uname -r)
Hardware: M5Stack Cardputer Zero (CM0)
Arch: armhf
RELEASE

# Mark first-boot as complete — disable the service
systemctl disable first-boot.service 2>/dev/null || true

echo ""
echo "[+] First boot setup complete."
echo "[+] Press Fn+Tab for TUI, or run: cyber_launcher"
echo ""
FIRSTBOOT
    chmod +x "${BIN}/first-boot"
fi

# Enable first-boot service
on_chroot << EOF
systemctl enable first-boot.service
EOF

echo "[zeroday] First-boot service configured."