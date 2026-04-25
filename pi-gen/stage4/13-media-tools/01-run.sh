#!/bin/bash -e
set -euo pipefail
# Install media playback scripts

install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

cat > "${ROOTFS_DIR}/usr/local/bin/webradio-danish" << 'EOF'
#!/bin/bash
# Danish WebRadio via ffplay

STATION=$1

case "${STATION}" in
    "DR_P1")
        URL="https://drradio1-lh.akamaihd.net/i/p1_9@143503/master.m3u8"
        ;;
    "DR_P3")
        URL="https://drradio3-lh.akamaihd.net/i/p3_9@143506/master.m3u8"
        ;;
    "NOVA")
        URL="https://stream.bauermedia.fi/nova/nova_64.aac"
        ;;
    "POPFM")
        URL="https://stream.bauermedia.fi/popfm/popfm_64.aac"
        ;;
    *)
        echo "Usage: webradio-danish [DR_P1|DR_P3|NOVA|POPFM]"
        exit 1
        ;;
esac

echo "Playing ${STATION}..."
# ffplay in background, no video, low latency for streaming
ffplay -nodisp -autoexit -infbuf "${URL}" >/dev/null 2>&1 &
echo "$!" > /tmp/webradio.pid
EOF

chmod +x "${ROOTFS_DIR}/usr/local/bin/webradio-danish"

cat > "${ROOTFS_DIR}/usr/local/bin/music-player" << 'EOF'
#!/bin/bash
# Local music player via ffplay

DIR=${1:-"/opt/cardputer/music"}

if [ ! -d "${DIR}" ]; then
    echo "Directory ${DIR} not found!"
    exit 1
fi

# Build playlist — ffplay cannot play a directory directly
AUDIO_FILES=()
for ext in mp3 flac wav ogg aac m4a; do
    while IFS= read -r -d '' f; do
        AUDIO_FILES+=("$f")
    done < <(find "${DIR}" -maxdepth 1 -name "*.${ext}" -print0 2>/dev/null)
done

if [ ${#AUDIO_FILES[@]} -eq 0 ]; then
    echo "No audio files found in ${DIR}"
    exit 1
fi

# Shuffle the playlist
shuffled=$(printf '%s\n' "${AUDIO_FILES[@]}" | shuf)

echo "Playing ${#AUDIO_FILES[@]} tracks from ${DIR}..."
ffplay -nodisp -autoexit -loglevel quiet ${shuffled} >/dev/null 2>&1 &
echo "$!" > /tmp/music-player.pid
EOF

chmod +x "${ROOTFS_DIR}/usr/local/bin/music-player"

# Create default music directory
mkdir -p "${ROOTFS_DIR}/opt/cardputer/music"
