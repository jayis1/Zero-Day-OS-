#!/bin/bash -e
# stage5/03-cleanup/01-run.sh — Final image optimization and cleanup

on_chroot << EOF

# ── Remove Kali force-overwrite config if it was left behind ──
rm -f /etc/apt/apt.conf.d/99force-overwrite 2>/dev/null || true

# ── Clean apt caches ──
apt-get clean
apt-get autoremove --purge -y 2>/dev/null || true
rm -rf /var/cache/apt/archives/*
rm -rf /var/lib/apt/lists/*

# ── Remove documentation to save space ──
rm -rf /usr/share/doc/*
rm -rf /usr/share/man/*
rm -rf /usr/share/locale/*
rm -rf /usr/share/info/*

# ── Remove Python cache files (scoped to avoid traversing host mounts) ──
find /usr /opt /var /etc -name "*.pyc" -delete 2>/dev/null || true
find /usr /opt /var /etc -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true

# ── Remove editor backup files ──
find /usr /opt /var /etc -name "*~" -delete 2>/dev/null || true
find /usr /opt /var /etc -name "*.bak" -delete 2>/dev/null || true

# ── Clear logs ──
find /var/log -type f -name "*.log" -delete 2>/dev/null || true
find /var/log -type f -name "*.gz" -delete 2>/dev/null || true
find /var/log -type f -name "*.old" -delete 2>/dev/null || true

# ── Remove swap file if present (don't swapoff — this is a chroot, not running system) ──
rm -f /var/swap
# Remove swap entry from fstab if present
sed -i '/swap/d' /etc/fstab 2>/dev/null || true

# ── Set root locale ──
echo "LANG=en_US.UTF-8" > /etc/default/locale

# ── Set DHCP timeout (fast boot) ──
if [ -f /etc/dhcpcd.conf ]; then
    echo "timeout 10" >> /etc/dhcpcd.conf
fi

EOF

# ── Write the MOTD ──
if [ -f "${BASE_DIR}/../configs/motd/motd" ]; then
    cp "${BASE_DIR}/../configs/motd/motd" "${ROOTFS_DIR}/etc/motd"
else
    cat > "${ROOTFS_DIR}/etc/motd" << 'MOTD'
╔══════════════════════════════════════════════════╗
║                                                  ║
║   ZERO-DAY OS  v0.1-pre                         ║
║   M5Stack Cardputer Zero (CM0)                  ║
║                                                  ║
║   WiFi: OFF  |  BT: OFF  |  Eth: DOWN           ║
║   All radios blocked at boot (stealth)          ║
║                                                  ║
║   Fn+Tab  TUI    Fn+P  Panic    Fn+O  OpenCode  ║
║   Fn+N    Nmap   Fn+B  BT Scan  Fn+S  Shell    ║
║   Fn+W    WiFi   Fn+C  Camera   Fn+I  IR       ║
║   Fn+D    Dongle Fn+A  Ask AI                   ║
║                                                  ║
║   ⚠  AUTHORIZED USE ONLY — Unauthorized access   ║
║   on networks you don't own is illegal.          ║
╚══════════════════════════════════════════════════╝
MOTD
fi

echo "[zeroday] Cleanup complete. Image ready for deployment."