#!/bin/bash -e
set -euo pipefail
# stage4/05-reverse-shell-kit/01-run.sh — Install reverse shell tools and generators

BIN="${ROOTFS_DIR}/usr/local/bin"

# Install reverse shell scripts from the project
for script in revshell-gen revshell-listen revshell-stabilize; do
    if [ -f "${BASE_DIR}/../scripts/reverse/${script}" ]; then
        cp "${BASE_DIR}/../scripts/reverse/${script}" "${BIN}/${script}"
        chmod +x "${BIN}/${script}"
        echo "[zeroday] Installed: ${script}"
    else
        echo "[zeroday] WARNING: Missing script: ${script}"
    fi
done

# Install netcat and socat (should already be from network-tools, but ensure)
on_chroot << EOF
apt-get install -y --no-install-recommends netcat-openbsd socat 2>/dev/null || true
EOF

echo "[zeroday] Reverse shell kit installed."