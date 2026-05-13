#!/bin/bash
# pi-gen/postrun.sh — Post-build tasks for ZERO-DAY OS
# This runs after the image is complete

echo "╔══════════════════════════════════════════════════╗"
echo "║  ZERO-DAY OS BUILD COMPLETE                      ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Image: ${DEPLOY_DIR}/${IMG_DATE}-${IMG_NAME}.img"
echo ""
echo "Next steps:"
echo "  1. Flash to microSD: dd if=${IMG_NAME}.img of=/dev/sdX bs=4M status=progress"
echo "  2. Insert into Cardputer Zero"
echo "  3. Login: root / zeroday"
echo "  4. Change password: passwd"
echo "  5. (Optional) Install RTL8821CU: dongle-setup install"
echo "  6. (Optional) Connect expansion modules: PN532, CC1101, Meshtastic"
echo ""