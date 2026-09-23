#!/bin/bash -e
set -euo pipefail
# stage3/12-security-hardening/01-run.sh — Security hardening pass
# Zero-Day OS v4.6 — target: zero-day exploit resistance
# Covers: sysctl hardening, nftables firewall, apparmor, ssh lockdown,
#         encrypted loot partition setup, attack surface reduction

echo "[security] Starting Zero-Day OS security hardening..."

# ─── 1. sysctl kernel hardening ────────────────────────────────────────────

cat > "${ROOTFS_DIR}/etc/sysctl.d/99-zeroday-hardening.conf" << 'SYSCTL'
# Zero-Day OS — kernel sysctl security hardening

# --- Kernel self-protection ---
# Restrict kernel pointer exposure in /proc (requires CAP_SYSLOG)
kernel.kptr_restrict = 2
# Restrict dmesg to root (prevents info leaks pre-exploit)
kernel.dmesg_restrict = 1
# Disable sysrq (prevents trivial memory dump / reboot tricks)
kernel.sysrq = 0
# Disable core dumps for setuid programs
fs.suid_dumpable = 0
# Restrict perf events to CAP_SYS_ADMIN
kernel.perf_event_paranoid = 3
# Disable BPF JIT spray via unprivileged bpf(2)
kernel.unprivileged_bpf_disabled = 1
# Harden BPF JIT compiler
net.core.bpf_jit_harden = 2
# Prevent ptrace of processes not descended from us (Yama LSM)
kernel.yama.ptrace_scope = 1
# Randomize virtual address space (ASLR)
kernel.randomize_va_space = 2
# Restrict userfaultfd to privileged users (CVE-2022-2590 mitigations)
vm.unprivileged_userfaultfd = 0
# Disable kexec_load (prevents loading unsigned kernels at runtime)
kernel.kexec_load_disabled = 1
# Protect kernel from being mapped executable from userspace
vm.mmap_min_addr = 65536

# --- Network hardening ---
# Prevent SYN flood
net.ipv4.tcp_syncookies = 1
# Block source-routed packets (routing attack prevention)
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0
net.ipv6.conf.all.accept_source_route = 0
# Reject ICMP redirects (MITM prevention)
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv6.conf.all.accept_redirects = 0
# Enable reverse path filtering (prevents IP spoofing)
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1
# Ignore ICMP broadcasts (Smurf attack)
net.ipv4.icmp_echo_ignore_broadcasts = 1
# Ignore bogus error responses
net.ipv4.icmp_ignore_bogus_error_responses = 1
# Log spoofed/source-routed packets
net.ipv4.conf.all.log_martians = 1
# Disable IPv6 router advertisements (unless WiFi needs it)
net.ipv6.conf.all.accept_ra = 0
net.ipv6.conf.default.accept_ra = 0
# TCP timestamps can leak uptime — disable
net.ipv4.tcp_timestamps = 0
# Increase TCP FIN-WAIT-2 timeout to reduce half-open state abuse
net.ipv4.tcp_fin_timeout = 15

# --- Filesystem hardening ---
# Restrict hardlink/symlink following (TOCTOU mitigation)
fs.protected_hardlinks = 1
fs.protected_symlinks = 1
# Restrict FIFO and regular file opens in world-writable sticky dirs
fs.protected_fifos = 2
fs.protected_regular = 2
SYSCTL

# ─── 2. nftables default-deny firewall ─────────────────────────────────────

mkdir -p "${ROOTFS_DIR}/etc/nftables.d"

cat > "${ROOTFS_DIR}/etc/nftables.conf" << 'NFT'
#!/usr/sbin/nft -f
# Zero-Day OS — nftables default-deny firewall

flush ruleset

table inet zeroday {
    # Allowed inbound services (operator-toggled at runtime)
    set allowed_tcp_in {
        type inet_service
        # Add ports here with: nft add element inet zeroday allowed_tcp_in { 22 }
    }

    chain input {
        type filter hook input priority filter; policy drop;

        # Loopback always accepted
        iifname "lo" accept

        # Established / related connections
        ct state { established, related } accept

        # ICMP (ping replies) — rate-limited
        ip protocol icmp icmp type { echo-request } limit rate 5/second accept
        ip6 nexthdr icmpv6 icmpv6 type { echo-request } limit rate 5/second accept

        # mDNS for local service discovery (restrict to LAN)
        udp dport 5353 ip saddr 224.0.0.0/4 accept

        # Explicitly allowed inbound TCP
        tcp dport @allowed_tcp_in accept

        # Log and drop everything else
        log prefix "[zd-fw-drop] " flags all drop
    }

    chain forward {
        type filter hook forward priority filter; policy drop;
    }

    chain output {
        type filter hook output priority filter; policy accept;

        # Block outbound connections from root except system procs (anti-rootkit)
        # meta skuid root meta skgid root ip daddr != 127.0.0.0/8 drop
        # NOTE: above commented — ZD offensive tools run as root, enable in defensive mode
    }
}
NFT

on_chroot << 'EOF'
# Enable nftables at boot
systemctl enable nftables 2>/dev/null || true

# Install nftables if not present (it should be via stage2 packages)
apt-get install -y --no-install-recommends nftables 2>/dev/null || true

# Disable legacy iptables in favour of nft
update-alternatives --set iptables /usr/sbin/iptables-nft 2>/dev/null || true
update-alternatives --set ip6tables /usr/sbin/ip6tables-nft 2>/dev/null || true
EOF

# ─── 3. SSH server lockdown ─────────────────────────────────────────────────

# SSHd drop-in: installed only if openssh-server is present (stage4 may install it)
mkdir -p "${ROOTFS_DIR}/etc/ssh/sshd_config.d"
cat > "${ROOTFS_DIR}/etc/ssh/sshd_config.d/99-zeroday-hardening.conf" << 'SSHD'
# Zero-Day OS SSH hardening — applied over /etc/ssh/sshd_config

# Only strong KEX / ciphers / MACs
KexAlgorithms curve25519-sha256@libssh.org,diffie-hellman-group16-sha512
Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com
MACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com

# Disable password auth — key only
PasswordAuthentication no
ChallengeResponseAuthentication no
KbdInteractiveAuthentication no

# Disable dangerous features
PermitRootLogin prohibit-password
X11Forwarding no
AllowTcpForwarding no
GatewayPorts no
PermitTunnel no
AllowAgentForwarding no
PermitEmptyPasswords no

# Reduce attack window
LoginGraceTime 20
MaxAuthTries 3
MaxSessions 2

# Idle timeout: 10 min client, 2 min server keepalive
ClientAliveInterval 600
ClientAliveCountMax 2
TCPKeepAlive no

# Limit to zeroday user group when set
# AllowGroups zeroday
SSHD

# ─── 4. AppArmor bootstrap ──────────────────────────────────────────────────

on_chroot << 'EOF'
apt-get install -y --no-install-recommends apparmor apparmor-utils apparmor-profiles 2>/dev/null || \
    echo "[security] AppArmor package deferred — install manually"
systemctl enable apparmor 2>/dev/null || true
EOF

# ─── 5. Restrict /proc to owner-only via hidepid=2 ─────────────────────────

# Add proc hidepid=2 to fstab (requires gid=proc group in passwd)
if ! grep -q "hidepid" "${ROOTFS_DIR}/etc/fstab" 2>/dev/null; then
    echo "proc  /proc  proc  defaults,hidepid=2,gid=proc  0  0" \
        >> "${ROOTFS_DIR}/etc/fstab"
fi

on_chroot << 'EOF'
# Ensure proc group exists
getent group proc >/dev/null 2>&1 || groupadd -r proc
EOF

# ─── 6. Disable legacy / dangerous services ─────────────────────────────────

on_chroot << 'EOF'
# Services with known historical exploit surface — disable if present
for svc in rpcbind rpc-statd nfs-server cups bluetooth hciuart \
           exim4 sendmail postfix telnetd xinetd inetd; do
    systemctl disable "$svc" 2>/dev/null || true
    systemctl mask   "$svc" 2>/dev/null || true
done

# Remove historically vulnerable packages if installed
apt-get purge -y --auto-remove telnet rsh-client rsh-server \
    nis yp-tools tftp 2>/dev/null || true
EOF

# ─── 7. Restrict SUID/SGID bit surface ──────────────────────────────────────

# Applied at first-boot by zeroday-secureboot script (we can't strip inside
# chroot safely without knowing final package set; the script scans at runtime)
cat > "${ROOTFS_DIR}/usr/local/bin/zeroday-suid-audit" << 'SUIDAUDIT'
#!/bin/bash
# zeroday-suid-audit — find and report SUID/SGID binaries
# Run: zeroday-suid-audit [--strip-sgid]

STRIP=${1:-""}
echo "[*] Zero-Day OS SUID/SGID Audit"
echo ""

SUID_LIST=$(find / -xdev \( -perm -4000 -o -perm -2000 \) -type f 2>/dev/null | sort)
COUNT=$(echo "$SUID_LIST" | grep -c . 2>/dev/null || echo 0)

echo "[*] Found $COUNT SUID/SGID binaries:"
echo ""
while IFS= read -r f; do
    PERMS=$(stat -c '%A %U:%G' "$f" 2>/dev/null)
    echo "  $PERMS  $f"
done <<< "$SUID_LIST"

echo ""

# Known-safe list: binaries that legitimately need SUID
WHITELIST=(
    /usr/bin/sudo
    /usr/bin/su
    /usr/bin/passwd
    /usr/bin/newgrp
    /usr/bin/gpasswd
    /usr/bin/chsh
    /usr/bin/chfn
    /usr/bin/mount
    /usr/bin/umount
    /usr/bin/pkexec
    /usr/sbin/pam_timestamp_check
    /usr/lib/openssh/ssh-keysign
    /usr/lib/dbus-1.0/dbus-daemon-launch-helper
)

echo "[*] Binaries NOT in whitelist (review these):"
SUSPICIOUS=0
while IFS= read -r f; do
    WL=0
    for w in "${WHITELIST[@]}"; do [ "$f" = "$w" ] && WL=1 && break; done
    if [ $WL -eq 0 ]; then
        echo "  [!] $f"
        SUSPICIOUS=$((SUSPICIOUS+1))
    fi
done <<< "$SUID_LIST"

echo ""
if [ $SUSPICIOUS -eq 0 ]; then
    echo "[+] All SUID/SGID binaries are whitelisted."
else
    echo "[!] $SUSPICIOUS non-whitelisted SUID/SGID binaries found."
    echo "    Review and remove SUID bit with: chmod u-s /path/to/binary"
fi

if [ "$STRIP" = "--strip-sgid" ]; then
    echo ""
    echo "[*] Stripping SGID from non-essential binaries..."
    SGID_STRIP=(
        /usr/bin/bsd-write
        /usr/bin/wall
        /usr/bin/crontab
        /usr/sbin/pam_extrausers_chkpwd
    )
    for b in "${SGID_STRIP[@]}"; do
        [ -f "$b" ] && chmod g-s "$b" && echo "  [-] stripped SGID: $b" || true
    done
fi
SUIDAUDIT
chmod 755 "${ROOTFS_DIR}/usr/local/bin/zeroday-suid-audit"

# ─── 8. Kernel command-line hardening flags ─────────────────────────────────

CMDLINE_FILE="${ROOTFS_DIR}/boot/cmdline.txt"
if [ -f "$CMDLINE_FILE" ]; then
    # Read current cmdline
    CMDLINE=$(cat "$CMDLINE_FILE")
    # Append security params if not already present
    for PARAM in \
        "apparmor=1" \
        "security=apparmor" \
        "init_on_alloc=1" \
        "init_on_free=1" \
        "slab_nomerge" \
        "page_alloc.shuffle=1" \
        "pti=on" \
        "vsyscall=none" \
        "debugfs=off" \
        "oops=panic" \
        "lsm=lockdown,yama,apparmor,bpf" \
        "lockdown=integrity"; do
        # Extract param name (before =)
        PNAME="${PARAM%%=*}"
        if ! echo "$CMDLINE" | grep -qw "$PNAME"; then
            CMDLINE="${CMDLINE} ${PARAM}"
        fi
    done
    echo "$CMDLINE" > "$CMDLINE_FILE"
    echo "[security] cmdline.txt hardening params applied."
fi

# ─── 9. Umask hardening ─────────────────────────────────────────────────────

# Global default: no world-readable new files
if ! grep -q "umask 027" "${ROOTFS_DIR}/etc/profile" 2>/dev/null; then
    echo "umask 027" >> "${ROOTFS_DIR}/etc/profile"
fi
if ! grep -q "umask 027" "${ROOTFS_DIR}/etc/bash.bashrc" 2>/dev/null; then
    echo "umask 027" >> "${ROOTFS_DIR}/etc/bash.bashrc"
fi

# ─── 10. Coredump disable ───────────────────────────────────────────────────

mkdir -p "${ROOTFS_DIR}/etc/security/limits.d"
cat > "${ROOTFS_DIR}/etc/security/limits.d/99-zeroday-coredump.conf" << 'EOF'
# Disable coredumps system-wide (prevent memory content exposure)
*  hard  core  0
*  soft  core  0
EOF

cat > "${ROOTFS_DIR}/etc/sysctl.d/99-zeroday-coredump.conf" << 'EOF'
# Disable coredump creation
kernel.core_pattern = /dev/null
fs.suid_dumpable = 0
EOF

echo "[security] Zero-Day OS security hardening complete."
