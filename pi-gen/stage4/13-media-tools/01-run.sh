#!/bin/bash -e
# Install media playback scripts

install -m 755 -d "${ROOTFS_DIR}/usr/local/bin"

cat > "${ROOTFS_DIR}/usr/local/bin/webradio-danish" << 'EOF'
#!/bin/bash
# Danish WebRadio via mpv

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
# mpv in background, no video, low cache for quick start
mpv --no-video --profile=low-latency "${URL}" &
echo $! > /tmp/webradio.pid
EOF

chmod +x "${ROOTFS_DIR}/usr/local/bin/webradio-danish"

cat > "${ROOTFS_DIR}/usr/local/bin/music-player" << 'EOF'
#!/bin/bash
# Local music player via mpv

DIR=${1:-"/opt/cardputer/music"}

if [ ! -d "${DIR}" ]; then
    echo "Directory ${DIR} not found!"
    exit 1
fi

echo "Playing music from ${DIR}..."
mpv --no-video --shuffle "${DIR}"/* &
echo $! > /tmp/music-player.pid
EOF

chmod +x "${ROOTFS_DIR}/usr/local/bin/music-player"

# Create default music directory
mkdir -p "${ROOTFS_DIR}/opt/cardputer/music"
