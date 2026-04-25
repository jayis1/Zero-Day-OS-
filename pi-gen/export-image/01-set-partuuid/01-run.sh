#!/bin/bash -e
# pi-gen/export-image/01-set-partuuid/01-run.sh — Set partition UUIDs
# NOTE: After the image is created by prerun.sh, there is no rootfs to
# chroot into. The PARTUUID was already set during image creation in prerun.sh.
# This script is a no-op since the image is already finalized.

# If EXPORT_ROOTFS_DIR exists and has a rootfs, we could modify it.
# But typically by this point, the rootfs has already been rsync'd into the image.
if [ -d "${EXPORT_ROOTFS_DIR}" ] && [ -d "${EXPORT_ROOTFS_DIR}/etc" ]; then
    IMG_FILE="${STAGE_WORK_DIR}/image-${IMG_NAME}-${IMG_SUFFIX}"
    if [ -f "${IMG_FILE}.img" ]; then
        # Use sgdisk to randomize GUIDs if available (makes each image unique)
        if command -v sgdisk >/dev/null 2>&1; then
            sgdisk -G "${IMG_FILE}.img" 2>/dev/null || true
        fi
    fi
fi