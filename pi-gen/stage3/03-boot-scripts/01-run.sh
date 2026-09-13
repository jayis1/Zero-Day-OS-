#!/bin/bash -e
set -euo pipefail
# stage3/03-boot-scripts/01-run.sh — Install boot scripts and systemd services

BIN="${ROOTFS_DIR}/usr/local/bin"
SYSTEMD="${ROOTFS_DIR}/etc/systemd/system"
SCRIPT_SRC="${PROJECT_ROOT}/scripts"

# Install all system scripts
mkdir -p "${BIN}"

for script in panic zeroday-boot zeroday-bootanim first-boot power-mode tamper-watch \
    cardputer-wifi-setup cardputer-wifi-toggle stealth-backlight-toggle usb-gadget-mode \
    mac-rotate loot-organize opencode-session opencode-ask device-lock webui webui-toggle; do
    if [ -f "${SCRIPT_SRC}/system/${script}" ]; then
        cp "${SCRIPT_SRC}/system/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# Install all hacking scripts
for category in wifi network bluetooth reverse ir camera subghz nfc mesh dongle hardware; do
    if [ -d "${SCRIPT_SRC}/${category}" ]; then
        for script in "${SCRIPT_SRC}/${category}"/*; do
            if [ -f "$script" ]; then
                name=$(basename "$script")
                cp "$script" "${BIN}/${name}"
                chmod +x "${BIN}/${name}"
                echo "[zeroday] Installed: ${name}"
            fi
        done
    fi
done

# Install systemd services
mkdir -p "${SYSTEMD}"

# zeroday-boot.service
cat > "${SYSTEMD}/zeroday-boot.service" << 'EOF'
[Unit]
Description=ZERO-DAY OS Boot Orchestration
After=multi-user.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/local/bin/zeroday-boot

[Install]
WantedBy=multi-user.target
EOF

# panic.service
cat > "${SYSTEMD}/panic.service" << 'EOF'
[Unit]
Description=ZERO-DAY OS Emergency Kill+Wipe
DefaultDependencies=no

[Service]
Type=oneshot
ExecStart=/usr/local/bin/panic
EOF

# tamper-watch.service
cat > "${SYSTEMD}/tamper-watch.service" << 'EOF'
[Unit]
Description=ZERO-DAY OS Tamper Detection (BMI270 IMU)
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/local/bin/tamper-watch
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable services
on_chroot << EOF
systemctl enable zeroday-boot.service
systemctl enable tamper-watch.service
EOF

# Install additional systemd services from project configs
for svc in webui.service ragnar.service; do
    if [ -f "${PROJECT_ROOT}/configs/systemd/${svc}" ]; then
        cp "${PROJECT_ROOT}/configs/systemd/${svc}" "${SYSTEMD}/${svc}"
        echo "[zeroday] Installed service: ${svc}"
    fi
done

# zeroday-gui and zeroday-tui are created by stage4
# Disable WebUI by default (exposes HTTP — enable manually with: systemctl enable webui)
on_chroot << EOF
systemctl disable webui.service 2>/dev/null || true
systemctl disable ragnar.service 2>/dev/null || true
EOF

# Install config files
CONF_DIR="${PROJECT_ROOT}/configs"

# Bash configuration
if [ -f "${CONF_DIR}/bash/.bashrc" ]; then
    cp "${CONF_DIR}/bash/.bashrc" "${ROOTFS_DIR}/root/.bashrc"
    echo "[zeroday] Installed: .bashrc"
fi

# MOTD
if [ -f "${CONF_DIR}/motd/motd" ]; then
    cp "${CONF_DIR}/motd/motd" "${ROOTFS_DIR}/etc/motd"
    echo "[zeroday] Installed: motd"
fi

# Network configuration
mkdir -p "${ROOTFS_DIR}/etc/systemd/network"
for netfile in 20-wired.network 30-wireless.network 40-dongle.network; do
    if [ -f "${CONF_DIR}/network/${netfile}" ]; then
        cp "${CONF_DIR}/network/${netfile}" "${ROOTFS_DIR}/etc/systemd/network/${netfile}"
        echo "[zeroday] Installed: ${netfile}"
    fi
done

# Wayland/cage environment
mkdir -p "${ROOTFS_DIR}/etc/xdg/cage"
if [ -f "${CONF_DIR}/wayland/cage.env" ]; then
    cp "${CONF_DIR}/wayland/cage.env" "${ROOTFS_DIR}/etc/xdg/cage/cage.env"
    echo "[zeroday] Installed: cage.env"
fi

# WebUI HTML dashboard
mkdir -p "${ROOTFS_DIR}/opt/cardputer/webui"
if [ -f "${CONF_DIR}/webui/index.html" ]; then
    cp "${CONF_DIR}/webui/index.html" "${ROOTFS_DIR}/opt/cardputer/webui/index.html"
    echo "[zeroday] Installed: webui/index.html"
fi

# Create loot directories
mkdir -p "${ROOTFS_DIR}/opt/cardputer/loot"/{creds,recon,general,exfil,uploads}
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config"/{c2,doh,tunnels,captive,webui,exfil-dns}
mkdir -p "${ROOTFS_DIR}/opt/cardputer/roms" 2>/dev/null || true
mkdir -p "${ROOTFS_DIR}/opt/cardputer/music" 2>/dev/null || true

echo "[zeroday] Boot scripts and services installed."