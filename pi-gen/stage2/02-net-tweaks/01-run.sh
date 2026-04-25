#!/bin/bash -e
# stage2/02-net-tweaks/01-run.sh — Network configuration for ZERO-DAY OS

# Use systemd-networkd instead of NetworkManager
on_chroot << EOF
systemctl enable systemd-networkd
systemctl enable systemd-resolved 2>/dev/null || true
systemctl disable NetworkManager 2>/dev/null || true
systemctl disable networking 2>/dev/null || true
EOF

# Configure wired ethernet with DHCP
mkdir -p "${ROOTFS_DIR}/etc/systemd/network"
cat > "${ROOTFS_DIR}/etc/systemd/network/20-wired.network" << 'EOF'
[Match]
Name=eth0

[Network]
DHCP=yes

[DHCPv4]
RouteMetric=10
EOF

# Configure wlan0 (built-in WiFi) — managed mode, DHCP
cat > "${ROOTFS_DIR}/etc/systemd/network/20-wireless.network" << 'EOF'
[Match]
Name=wlan0

[Network]
DHCP=yes

[DHCPv4]
RouteMetric=20
EOF

# Configure wlan1 (RTL8821CU dongle) — usually monitor mode, but DHCP when managed
cat > "${ROOTFS_DIR}/etc/systemd/network/25-dongle.network" << 'EOF'
[Match]
Name=wlan1

[Network]
DHCP=yes

[DHCPv4]
RouteMetric=30
EOF

# Block all wireless radios at boot (stealth default)
on_chroot << EOF
rfkill block all 2>/dev/null || true
EOF

# DNS configuration for systemd-resolved — DO NOT symlink resolv.conf here!
# During build, /etc/resolv.conf must remain a static file for chroot DNS to work.
# The symlink to /run/systemd/resolve/stub-resolv.conf will be created at first boot
# by a systemd service (see stage5/00-first-boot).
# Leaving the debootstrap-provided /etc/resolv.conf intact for the rest of the build.