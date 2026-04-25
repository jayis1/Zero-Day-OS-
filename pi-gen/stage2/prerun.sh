#!/bin/bash -e
# stage2/prerun.sh — Copy previous stage rootfs

if [ ! -d "${ROOTFS_DIR}" ]; then
	copy_previous
fi