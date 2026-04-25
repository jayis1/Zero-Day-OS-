#!/bin/bash -e
# stage3/00-configure-base/01-run.sh — ZERO-DAY OS base system configuration

# Set hostname
echo "zeroday" > "${ROOTFS_DIR}/etc/hostname"
echo "127.0.1.1  zeroday" >> "${ROOTFS_DIR}/etc/hosts"

# Set root password
# Generate hash outside chroot and write directly to shadow file
# This avoids PAM issues inside the chroot environment
ZERODAY_HASH=$(openssl passwd -6 "zeroday")
sed -i "s|^root:[^:]*:|root:${ZERODAY_HASH}:|" "${ROOTFS_DIR}/etc/shadow"

# Create operator user with same password
on_chroot << 'OPERATOR'
useradd -m -s /bin/bash operator 2>/dev/null || true
OPERATOR

# Set operator password via shadow file (same approach as root)
OPERATOR_LINE=$(grep "^operator:" "${ROOTFS_DIR}/etc/shadow" 2>/dev/null || true)
if [ -n "${OPERATOR_LINE}" ]; then
    sed -i "s|^operator:[^:]*:|operator:${ZERODAY_HASH}:|" "${ROOTFS_DIR}/etc/shadow"
fi

# Set up passwordless sudo for operator
echo "operator ALL=(ALL) NOPASSWD:ALL" > "${ROOTFS_DIR}/etc/sudoers.d/operator"
chmod 440 "${ROOTFS_DIR}/etc/sudoers.d/operator"

# Set locale
on_chroot << EOF
if [ -f /etc/locale.gen ]; then
    sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen
    locale-gen
    update-locale LANG=en_US.UTF-8
else
    # locales package may not be installed yet
    apt-get install -y --no-install-recommends locales 2>/dev/null || true
    if [ -f /etc/locale.gen ]; then
        sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen
        locale-gen
        update-locale LANG=en_US.UTF-8
    fi
fi
EOF

# Set timezone
on_chroot << EOF
echo "UTC" > /etc/timezone
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
EOF

# System tuning — minimize SD card writes
cat > "${ROOTFS_DIR}/etc/sysctl.d/99-zeroday.conf" << 'EOF'
# ZERO-DAY OS kernel tuning
vm.swappiness=1
vm.dirty_ratio=10
vm.dirty_background_ratio=5
vm.dirty_writeback_centisecs=1500
vm.dirty_expire_centisecs=3000

# Network hardening
net.ipv4.tcp_syncookies=1
net.ipv4.conf.all.rp_filter=1
net.ipv4.conf.default.rp_filter=1
net.ipv4.icmp_echo_ignore_broadcasts=1
net.ipv4.conf.all.accept_redirects=0
net.ipv4.conf.default.accept_redirects=0

# Shared memory protection
kernel.randomize_va_space=2
EOF

# Mount /tmp, /var/log, /var/tmp as tmpfs (RAM disks)
cat >> "${ROOTFS_DIR}/etc/fstab" << 'EOF'
# ZERO-DAY OS — tmpfs mounts to reduce SD card writes
tmpfs /tmp         tmpfs defaults,noatime,nosuid,size=64m 0 0
tmpfs /var/log     tmpfs defaults,noatime,nosuid,size=16m 0 0
tmpfs /var/tmp     tmpfs defaults,noatime,nosuid,size=16m 0 0
EOF

# Disable unnecessary services
on_chroot << EOF
systemctl disable bluetooth 2>/dev/null || true
systemctl disable ModemManager 2>/dev/null || true
systemctl disable avahi-daemon 2>/dev/null || true
systemctl disable cups 2>/dev/null || true
systemctl disable triggerhappy 2>/dev/null || true
systemctl disable wifi-country 2>/dev/null || true
systemctl disable apt-daily.timer 2>/dev/null || true
systemctl disable apt-daily-upgrade.timer 2>/dev/null || true
systemctl disable man-db.timer 2>/dev/null || true
EOF

# Create ZERO-DAY OS directory structure
mkdir -p "${ROOTFS_DIR}/opt/cardputer"
mkdir -p "${ROOTFS_DIR}/opt/cardputer/"{handshakes,pmkid,payloads,workspace}
mkdir -p "${ROOTFS_DIR}/opt/cardputer/loot/"{recon,bt,ble,ir,cam,rf,nfc,captive,screenshot}
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config/"{attack-profiles,wordlists,ir_codes,opencode,meshtastic,captive}
chmod 700 "${ROOTFS_DIR}/opt/cardputer"
chmod 700 "${ROOTFS_DIR}/opt/cardputer/loot"

# Write ZERO-DAY OS release info
cat > "${ROOTFS_DIR}/etc/zeroday-release" << 'EOF'
ZERO-DAY OS v0.1-pre
Build: DATE_PLACEHOLDER
Kernel: PLACEHOLDER
Hardware: M5Stack Cardputer Zero (CM0)
Arch: armhf
EOF

# Configure bash
cat > "${ROOTFS_DIR}/etc/skel/.bashrc" << 'BASHRC'
# ~/.bashrc — ZERO-DAY OS
export PS1='\[\033[01;31m\]zero\[\033[01;33m\]@\[\033[01;32m\]day\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]# '
export PATH="/usr/local/bin:$PATH"
export HISTSIZE=100
export HISTCONTROL=ignoreboth

# Aliases
alias ll='ls -la'
alias la='ls -a'
alias l='ls -CF'
alias cls='clear'
alias ..='cd ..'
alias ...='cd ../..'

# ZERO-DAY OS quick commands
alias scan='sudo wifi-scan'
alias deauth='sudo wifi-deauth'
alias capture='sudo wifi-handshake'
alias crack='sudo wifi-crack'
alias nmap-quick='sudo net-quickscan'
alias btscan='sudo bt-scan'
alias panic='sudo panic'
alias battery='cardputer-battery'
alias dongle='dongle-setup status'
alias tui='cyber_launcher'
alias opencode='opencode-session'
alias mesh='mesh-chat'

# Turn off flow control (so Ctrl+S works in programs)
stty -ixon 2>/dev/null || true
BASHRC

# Copy bashrc to root home
cp "${ROOTFS_DIR}/etc/skel/.bashrc" "${ROOTFS_DIR}/root/.bashrc"

# Auto-login root on tty1
mkdir -p "${ROOTFS_DIR}/etc/systemd/system/getty@tty1.service.d"
cat > "${ROOTFS_DIR}/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I $TERM
EOF