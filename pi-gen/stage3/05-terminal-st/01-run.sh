#!/bin/bash -e
set -euo pipefail
# stage3/05-terminal-st/01-run.sh — Configure st terminal and fbterm fallback

# Configure st terminal (if compiled from source) or use stterm
# stterm is the Debian package for st (simple terminal)
mkdir -p "${ROOTFS_DIR}/etc/st"
cat > "${ROOTFS_DIR}/etc/st/config.h" << 'STCONF'
/* ZERO-DAY OS st configuration — optimized for 1.9" 320x240 LCD */
static char *font = "monospace:size=8";
static int borderpx = 0;

/* Disable scrollback — tmux handles this */
static int allowaltpc = 1;

/* 256 color for performance */
static unsigned int defaultfg = 7;
static unsigned int defaultbg = 0;
static unsigned int defaultcs = 7;

/* Mouse — disabled, keyboard only */
static unsigned int mouseshape = XC_xterm;
static unsigned int mousefg = 7;
static unsigned int mousebg = 0;
STCONF

# Configure fbterm as fallback (no X11 needed — used in stealth power mode)
mkdir -p "${ROOTFS_DIR}/etc"
cat > "${ROOTFS_DIR}/etc/fbterm.conf" << 'FBTERM'
# ZERO-DAY OS fbterm configuration
# Used when Xorg is killed in stealth power mode
font-size=8
color-mode=256
cursor-shape=1
text-mode=0
FBTERM

# Set fbterm as setuid so it can access framebuffer
on_chroot << EOF
if command -v fbterm >/dev/null 2>&1; then
    chmod 4755 /usr/bin/fbterm 2>/dev/null || true
fi
EOF

echo "[zeroday] Terminal configuration installed."