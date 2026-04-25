#!/bin/bash -e
# stage2/01-sys-tweaks/01-run.sh — System tweaks

# Set hostname
echo "${TARGET_HOSTNAME}" > "${ROOTFS_DIR}/etc/hostname"

# Set timezone
ln -sf "/usr/share/zoneinfo/${TIMEZONE_DEFAULT}" "${ROOTFS_DIR}/etc/localtime"

# Set default locale
echo "LANG=${LOCALE_DEFAULT}" > "${ROOTFS_DIR}/etc/default/locale"

# Enable serial console (try both RPi serial devices)
on_chroot << EOF
systemctl enable serial-getty@ttyAMA0.service 2>/dev/null || true
systemctl enable serial-getty@ttyS0.service 2>/dev/null || true
EOF

# Disable unnecessary services
on_chroot << EOF
systemctl disable ModemManager 2>/dev/null || true
systemctl disable avahi-daemon 2>/dev/null || true
systemctl disable triggerhappy 2>/dev/null || true
systemctl disable cups 2>/dev/null || true
EOF