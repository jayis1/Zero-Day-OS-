#!/bin/bash -e
# stage4/09-wordlists-seclists/01-run.sh — Install SecLists (compressed)

on_chroot << EOF
apt-get -y -t kali-rolling install --no-install-recommends seclists 2>/dev/null || {
    echo "[zeroday] SecLists not available from Kali repos, downloading..."
    if command -v git >/dev/null 2>&1; then
        git clone --depth 1 https://github.com/danielmiessler/SecLists.git /opt/seclists 2>/dev/null || {
            echo "[zeroday] SecLists download failed — install manually later"
        }
    fi
}

# Trim SecLists to essential files only (save ~800MB on SD card)
if [ -d /opt/seclists ]; then
    SECLIST_DIR="/opt/seclists"
    WORDLIST_DIR="/opt/cardputer/config/wordlists"
    mkdir -p "\${WORDLIST_DIR}"

    # Copy essential wordlists
    cp "\${SECLIST_DIR}/Passwords/Default-Credentials/default-passwords.csv" "\${WORDLIST_DIR}/" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Passwords/Common-Credentials/best1050.txt" "\${WORDLIST_DIR}/" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Passwords/Common-Credentials/10-million-password-list-top-1000000.txt" "\${WORDLIST_DIR}/rockyou-top1m.txt" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Usernames/Names/names.txt" "\${WORDLIST_DIR}/" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Discovery/Web-Content/common.txt" "\${WORDLIST_DIR}/" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Discovery/Web-Content/raft-medium-directories.txt" "\${WORDLIST_DIR}/" 2>/dev/null || true
    cp "\${SECLIST_DIR}/Fuzzing/fuzz-Bo0o1.txt" "\${WORDLIST_DIR}/" 2>/dev/null || true

    # Remove full SecLists repo to save space
    rm -rf "\${SECLIST_DIR}"
fi
EOF

echo "[zeroday] Wordlists installed."