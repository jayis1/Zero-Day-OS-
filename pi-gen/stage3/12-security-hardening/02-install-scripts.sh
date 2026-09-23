#!/bin/bash -e
set -euo pipefail
# stage3/12-security-hardening/02-install-scripts.sh — Install security scripts

echo "[security] Installing Zero-Day OS security scripts..."

SCRIPTS_SRC="${STAGE_WORK_DIR}/../../scripts/system"

# Install scripts to /usr/local/bin
for script in zeroday-crypt zeroday-secureboot zeroday-security-check zeroday-suid-audit; do
    SRC="${SCRIPTS_SRC}/${script}"
    DST="${ROOTFS_DIR}/usr/local/bin/${script}"
    if [ -f "$SRC" ]; then
        install -m 755 "$SRC" "$DST"
        echo "  installed: /usr/local/bin/${script}"
    else
        echo "  [!] source not found: ${SRC} (skip)"
    fi
done

# Install systemd service for security-check at first boot
cat > "${ROOTFS_DIR}/etc/systemd/system/zeroday-security-check.service" << 'SVC'
[Unit]
Description=Zero-Day OS Security Check (first-boot validation)
After=multi-user.target
ConditionPathExists=!/var/lib/zeroday-security-check.done

[Service]
Type=oneshot
ExecStart=/usr/local/bin/zeroday-security-check --json
ExecStartPost=/usr/bin/touch /var/lib/zeroday-security-check.done
StandardOutput=journal
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
SVC

on_chroot << 'EOF'
systemctl enable zeroday-security-check.service 2>/dev/null || true
EOF

echo "[security] Security scripts installed."
