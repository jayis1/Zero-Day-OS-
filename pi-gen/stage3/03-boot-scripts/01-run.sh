#!/bin/bash -e
set -euo pipefail
# stage3/03-boot-scripts/01-run.sh — Install boot scripts and systemd services

BIN="${ROOTFS_DIR}/usr/local/bin"
SYSTEMD="${ROOTFS_DIR}/etc/systemd/system"
SCRIPT_SRC="${BASE_DIR}/../scripts"

# Install all system scripts
mkdir -p "${BIN}"

for script in panic zeroday-boot zeroday-bootanim first-boot power-mode tamper-watch \
    cardputer-wifi-setup cardputer-wifi-toggle stealth-backlight-toggle usb-gadget-mode \
    mac-rotate loot-organize opencode-session opencode-ask; do
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

echo "[zeroday] Boot scripts and services installed."