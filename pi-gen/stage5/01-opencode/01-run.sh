#!/bin/bash -e
# stage5/01-opencode/01-run.sh — Install OpenCode

BIN="${ROOTFS_DIR}/usr/local/bin"

# OpenCode is a Go/Rust binary — not a Python package.
# Try downloading the armhf binary, fall back to a stub.
on_chroot << 'OPENCODE_INSTALL'
# Determine architecture for binary download
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "armhf")

# Try GitHub release binary (armv7 = armhf on Linux)
for SUFFIX in "linux-armv7" "linux-armhf" "linux-arm"; do
    URL="https://github.com/opencode-ai/opencode/releases/latest/download/opencode-${SUFFIX}"
    if curl -sLf "${URL}" -o /usr/local/bin/opencode 2>/dev/null; then
        chmod +x /usr/local/bin/opencode
        echo "[zeroday] OpenCode installed: opencode-${SUFFIX}"
        break
    fi
done

# If no binary was downloaded, create a stub
if [ ! -x /usr/local/bin/opencode ]; then
    cat > /usr/local/bin/opencode << 'STUB'
#!/bin/sh
echo "OpenCode not installed (no armhf binary available)."
echo "Install manually from: https://github.com/opencode-ai/opencode/releases"
echo "Or try: pip3 install opencode-ai (Python wrapper)"
STUB
    chmod +x /usr/local/bin/opencode
    echo "[zeroday] OpenCode binary not available for armhf — created stub"
fi
OPENCODE_INSTALL

# Create workspace directory
mkdir -p "${ROOTFS_DIR}/opt/cardputer/workspace"
mkdir -p "${ROOTFS_DIR}/opt/cardputer/config/opencode"

echo "[zeroday] OpenCode installed."