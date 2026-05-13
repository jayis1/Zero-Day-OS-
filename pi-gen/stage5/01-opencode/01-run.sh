#!/bin/bash -e
set -euo pipefail
# stage5/01-opencode/01-run.sh — Install OpenCode CLI (aarch64)

BIN="${ROOTFS_DIR}/usr/local/bin"
OPENCODE_VERSION="v1.14.49"
OPENCODE_URL="https://github.com/anomalyco/opencode/releases/download/${OPENCODE_VERSION}/opencode-linux-arm64.tar.gz"

TMP="$(mktemp -d)"
echo "[zeroday] Downloading OpenCode ${OPENCODE_VERSION} for arm64..."

if curl -sLf "${OPENCODE_URL}" -o "${TMP}/opencode-linux-arm64.tar.gz" 2>/dev/null; then
    tar -xzf "${TMP}/opencode-linux-arm64.tar.gz" -C "${TMP}/" 2>/dev/null || true
    # Find the binary in the extracted archive
    OPENCODE_BIN="$(find "${TMP}" -maxdepth 2 -type f -name 'opencode' ! -name '*.sha256' 2>/dev/null | head -1)"
    if [ -n "${OPENCODE_BIN}" ] && [ -x "${OPENCODE_BIN}" ] ; then
        cp "${OPENCODE_BIN}" "${BIN}/opencode"
        chmod +x "${BIN}/opencode"
        echo "[zeroday] OpenCode ${OPENCODE_VERSION} installed successfully"
    else
        echo "[zeroday] WARNING: Could not find opencode binary in archive, creating stub"
        cat > "${BIN}/opencode" << 'STUB'
#!/bin/sh
echo "OpenCode not installed (binary extraction failed)."
echo "Install manually from: https://github.com/anomalyco/opencode/releases"
STUB
        chmod +x "${BIN}/opencode"
    fi
else
    echo "[zeroday] WARNING: Could not download OpenCode, creating stub"
    cat > "${BIN}/opencode" << 'STUB'
#!/bin/sh
echo "OpenCode not installed (download failed)."
echo "Install manually from: https://github.com/anomalyco/opencode/releases"
STUB
    chmod +x "${BIN}/opencode"
fi

rm -rf "${TMP}"

# Create workspace directory
mkdir -p "${ROOTFS_DIR}/opt/cardputer/workspace"
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config/opencode"

echo "[zeroday] OpenCode install complete."