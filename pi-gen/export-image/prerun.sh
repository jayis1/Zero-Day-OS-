#!/bin/bash -e
# pi-gen/export-image/prerun.sh — Create the final SD card image

IMG_FILE="${STAGE_WORK_DIR}/image-${IMG_NAME}-${IMG_SUFFIX}"

# Create image file (2GB for base + tool growth)
IMG_SIZE=3600  # MB — target for ~3.5GB image

# Ensure the work directory exists
mkdir -p "${STAGE_WORK_DIR}"

ensure_next_loopdev

# Create the disk image
truncate -s "${IMG_SIZE}M" "${IMG_FILE}.img"

# Partition the image
parted -s "${IMG_FILE}.img" mklabel msdos
parted -s "${IMG_FILE}.img" mkpart primary fat32 4MiB 512MiB
parted -s "${IMG_FILE}.img" mkpart primary ext4 512MiB 100%

# Set boot flag on first partition
parted -s "${IMG_FILE}.img" set 1 boot on

# Setup loop device
LOOP_DEV=$(losetup --show -fP "${IMG_FILE}.img")
echo "[zeroday] Using loop device: ${LOOP_DEV}"

# Wait for partitions to appear
sleep 2
ensure_loopdev_partitions "${LOOP_DEV}"

# Format boot partition (FAT32)
mkfs.vfat -F 32 -n BOOT "${LOOP_DEV}p1"

# Format root partition (ext4)
mkfs.ext4 -L ROOTFS "${LOOP_DEV}p2"

# Mount and copy rootfs
mkdir -p "${STAGE_WORK_DIR}/root" "${STAGE_WORK_DIR}/boot"
mount "${LOOP_DEV}p2" "${STAGE_WORK_DIR}/root"
mkdir -p "${STAGE_WORK_DIR}/root/boot"
mount "${LOOP_DEV}p1" "${STAGE_WORK_DIR}/root/boot"

# Copy rootfs
rsync -aHAXx "${EXPORT_ROOTFS_DIR}/" "${STAGE_WORK_DIR}/root/"

# Write partition UUIDs to fstab
PARTUUID_BOOT=$(blkid -o value -s PARTUUID "${LOOP_DEV}p1" 2>/dev/null || true)
PARTUUID_ROOT=$(blkid -o value -s PARTUUID "${LOOP_DEV}p2" 2>/dev/null || true)

if [ -n "${PARTUUID_BOOT}" ] && [ -n "${PARTUUID_ROOT}" ]; then
    cat > "${STAGE_WORK_DIR}/root/etc/fstab" << FSTAB
PARTUUID=${PARTUUID_BOOT}  /boot   vfat    defaults          0   2
PARTUUID=${PARTUUID_ROOT}  /       ext4    defaults,noatime  0   1

# ZERO-DAY OS — tmpfs mounts
tmpfs /tmp         tmpfs defaults,noatime,nosuid,size=64m 0 0
tmpfs /var/log     tmpfs defaults,noatime,nosuid,size=16m 0 0
tmpfs /var/tmp     tmpfs defaults,noatime,nosuid,size=16m 0 0
FSTAB
else
    # Fallback: use device paths if PARTUUID is unavailable (e.g., in Docker)
    cat > "${STAGE_WORK_DIR}/root/etc/fstab" << FSTAB
/dev/mmcblk0p1  /boot   vfat    defaults          0   2
/dev/mmcblk0p2  /       ext4    defaults,noatime  0   1

# ZERO-DAY OS — tmpfs mounts
tmpfs /tmp         tmpfs defaults,noatime,nosuid,size=64m 0 0
tmpfs /var/log     tmpfs defaults,noatime,nosuid,size=16m 0 0
tmpfs /var/tmp     tmpfs defaults,noatime,nosuid,size=16m 0 0
FSTAB
fi

# Write cmdline.txt
if [ -n "${PARTUUID_ROOT}" ]; then
    cat > "${STAGE_WORK_DIR}/root/boot/cmdline.txt" << CMDLINE
console=serial0,115200 console=tty1 root=PARTUUID=${PARTUUID_ROOT} rootfstype=ext4 fsck.repair=yes rootwait quiet
CMDLINE
else
    cat > "${STAGE_WORK_DIR}/root/boot/cmdline.txt" << CMDLINE
console=serial0,115200 console=tty1 root=/dev/mmcblk0p2 rootfstype=ext4 fsck.repair=yes rootwait quiet
CMDLINE
fi

# Unmount
umount "${STAGE_WORK_DIR}/root/boot" || true
umount "${STAGE_WORK_DIR}/root" || true

# Detach loop
losetup -d "${LOOP_DEV}" 2>/dev/null || true

# Compress image
echo "[zeroday] Compressing image..."
case "${DEPLOY_COMPRESSION}" in
    zip)
        pushd "${DEPLOY_DIR}"
        zip -"${COMPRESSION_LEVEL}" -r \
            "${IMG_DATE}-${IMG_NAME}-${IMG_SUFFIX}.zip" \
            "${IMG_FILE}.img"
        popd
        ;;
    gz)
        pigz -"${COMPRESSION_LEVEL}" \
            -c "${IMG_FILE}.img" > \
            "${DEPLOY_DIR}/${IMG_DATE}-${IMG_NAME}-${IMG_SUFFIX}.img.gz"
        ;;
    xz)
        xz -"${COMPRESSION_LEVEL}" \
            -c "${IMG_FILE}.img" > \
            "${DEPLOY_DIR}/${IMG_DATE}-${IMG_NAME}-${IMG_SUFFIX}.img.xz"
        ;;
    none|*)
        cp "${IMG_FILE}.img" "${DEPLOY_DIR}/"
        ;;
esac

echo "[zeroday] Image created: ${DEPLOY_DIR}/"
echo "[zeroday] Flash with: sudo dd if=zeroday-os.img of=/dev/sdX bs=4M status=progress conv=fsync"