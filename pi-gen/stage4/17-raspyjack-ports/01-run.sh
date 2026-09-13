#!/bin/bash -e
set -euo pipefail
# stage4/17-raspyjack-ports/01-run.sh
# Install Raspyjack-ported offensive tools for ZERO-DAY OS
# All scripts are pure bash targeting arm64 / Cardputer Zero constraints:
#   - No RPi.GPIO, no LCD_1in44 deps
#   - Python used only for inline parsing (no heavy frameworks)
#   - All apt installs are best-effort with graceful fallback

BIN="${ROOTFS_DIR}/usr/local/bin"
WORDLISTS="${ROOTFS_DIR}/opt/cardputer/config/wordlists"
mkdir -p "${WORDLISTS}"

# ── Install scripts ───────────────────────────────────────────────────────────

# WiFi
for script in wps-attack wifi-probe-dump wifi-karma-ap; do
    if [ -f "${PROJECT_ROOT}/scripts/wifi/${script}" ]; then
        install -m 755 "${PROJECT_ROOT}/scripts/wifi/${script}" "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# Network
for script in dhcp-attack vlan-hop snmp-scan http-probe; do
    if [ -f "${PROJECT_ROOT}/scripts/network/${script}" ]; then
        install -m 755 "${PROJECT_ROOT}/scripts/network/${script}" "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# Reverse / post-exploitation
for script in kerberoast pass-spray privesc-check post-harvest; do
    if [ -f "${PROJECT_ROOT}/scripts/reverse/${script}" ]; then
        install -m 755 "${PROJECT_ROOT}/scripts/reverse/${script}" "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# System
for script in engagement-clean; do
    if [ -f "${PROJECT_ROOT}/scripts/system/${script}" ]; then
        install -m 755 "${PROJECT_ROOT}/scripts/system/${script}" "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    fi
done
if [ -f "${PROJECT_ROOT}/scripts/wifi/wifi-autocrack" ]; then
    install -m 755 "${PROJECT_ROOT}/scripts/wifi/wifi-autocrack" "${BIN}/wifi-autocrack"
    echo "[zeroday] Installed: wifi-autocrack"
fi

# ── Wordlists ─────────────────────────────────────────────────────────────────

# wifi_top4800.txt — optimized WiFi password list from Raspyjack
if [ -f "${PROJECT_ROOT}/loot/wordlists/wifi_top4800.txt" ]; then
    install -m 644 "${PROJECT_ROOT}/loot/wordlists/wifi_top4800.txt" \
        "${WORDLISTS}/wifi_top4800.txt"
    echo "[zeroday] Installed wordlist: wifi_top4800.txt ($(wc -l < "${PROJECT_ROOT}/loot/wordlists/wifi_top4800.txt") words)"
fi

# ── Apt packages — arm64 compatible, best-effort ─────────────────────────────

on_chroot << 'EOF'
export DEBIAN_FRONTEND=noninteractive

echo "[zeroday] Installing Raspyjack-port dependencies..."

# WPS attack tools
apt-get -y install --no-install-recommends reaver 2>/dev/null \
    || apt-get -y -t kali-rolling install --no-install-recommends reaver 2>/dev/null \
    || echo "[zeroday] reaver not available — wps-attack will report missing"

# Packet capture / analysis (arm64 native)
apt-get -y install --no-install-recommends \
    tshark \
    tcpdump \
    2>/dev/null || echo "[zeroday] tshark/tcpdump install issue"

# Scapy — needed for dhcp-attack, vlan-hop (no RPi.GPIO required)
apt-get -y install --no-install-recommends python3-scapy 2>/dev/null \
    || pip3 install --break-system-packages scapy 2>/dev/null \
    || echo "[zeroday] scapy not installed — dhcp-attack/vlan-hop need it"

# SNMP tools
apt-get -y install --no-install-recommends snmp snmp-mibs-downloader 2>/dev/null \
    || echo "[zeroday] snmp tools install deferred"

# impacket — kerberoast, pass-the-hash
apt-get -y install --no-install-recommends python3-impacket 2>/dev/null \
    || pip3 install --break-system-packages impacket 2>/dev/null \
    || echo "[zeroday] impacket not installed — kerberoast needs it"

# smbclient — pass-spray smb mode, post-harvest
apt-get -y install --no-install-recommends smbclient 2>/dev/null \
    || echo "[zeroday] smbclient install deferred"

# sshpass — pass-spray ssh mode
apt-get -y install --no-install-recommends sshpass 2>/dev/null \
    || echo "[zeroday] sshpass install deferred"

# hashcat — wifi-autocrack tiers 2-4
# Note: hashcat on arm64 CPU-only (no GPU) is slow but works for small wordlists
apt-get -y install --no-install-recommends hashcat 2>/dev/null \
    || apt-get -y -t kali-rolling install --no-install-recommends hashcat 2>/dev/null \
    || echo "[zeroday] hashcat not available — wifi-autocrack will use aircrack-ng only"

# hashcat-utils — hcxpcapngtool for .cap -> .22000 conversion
apt-get -y -t kali-rolling install --no-install-recommends hashcat-utils 2>/dev/null \
    || echo "[zeroday] hashcat-utils not available — using cap2hccapx fallback"

# hcxdumptool / hcxtools — already in stage4/01-wifi-tools but ensure here
apt-get -y install --no-install-recommends hcxtools 2>/dev/null || true

# inotify-tools — for any file watching (optional, wifi-autocrack doesn't need it anymore)
apt-get -y install --no-install-recommends inotify-tools 2>/dev/null || true

echo "[zeroday] Raspyjack-port dependencies installed."
EOF

# ── udev: allow non-root tshark ──────────────────────────────────────────────

# Add root to wireshark group for tshark
on_chroot << 'EOF'
# Allow root to use tshark without sudo (it already is root, but for consistency)
groupadd -f wireshark 2>/dev/null || true
usermod -aG wireshark root 2>/dev/null || true

# Enable non-root capture for tshark (sets capabilities)
dpkg-reconfigure -p critical wireshark-common 2>/dev/null || true
EOF

# ── Loot directories ──────────────────────────────────────────────────────────

mkdir -p "${ROOTFS_DIR}/opt/cardputer/loot"/{wifi,recon,creds,general,rf,bt}
mkdir -p "${ROOTFS_DIR}/opt/cardputer/handshakes"
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config/wordlists"

echo "[zeroday] Raspyjack ports installed."
