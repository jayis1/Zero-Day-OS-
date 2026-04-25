#!/bin/bash -e
# stage4/00-kali-repos/01-run.sh — Add Kali Rolling repository with low priority
# Note: Kali armhf has limited package availability. Install failures are
# gracefully handled — packages that aren't available for armhf will be skipped.

# Add Kali repository
cat > "${ROOTFS_DIR}/etc/apt/sources.list.d/kali.list" << 'EOF'
deb http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware
EOF

# Kali gets low priority — only install when explicitly requested (-t kali-rolling)
cat > "${ROOTFS_DIR}/etc/apt/preferences.d/kali.pref" << 'EOF'
Package: *
Pin: release o=Kali
Pin-Priority: 50
EOF

# Note: --force-overwrite is NOT set globally. It is only applied
# per-package when installing from Kali via apt-get -t kali-rolling.
# This prevents Kali packages from silently clobbering Debian system files.

# Add Kali GPG key (best-effort — network may be unavailable)
on_chroot << 'EOF'
# Download Kali GPG key
if wget -qO /tmp/kali-key.asc https://archive.kali.org/archive-key.asc 2>/dev/null; then
    gpg --dearmor < /tmp/kali-key.asc > /usr/share/keyrings/kali-archive-keyring.gpg 2>/dev/null || true
    rm -f /tmp/kali-key.asc
fi

# Ensure keyring directory exists
mkdir -p /etc/apt/trusted.gpg.d/
cp /usr/share/keyrings/kali-archive-keyring.gpg /etc/apt/trusted.gpg.d/ 2>/dev/null || true

# Update apt indexes (best-effort — Kali mirrors can be slow/unreliable)
apt-get update -o Acquire::AllowInsecureRepository=true 2>/dev/null || true
apt-get update 2>/dev/null || true
EOF

echo "[zeroday] Kali Rolling repository added (priority 50)."