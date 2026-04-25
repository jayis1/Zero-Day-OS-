#!/bin/bash -e
# stage4/prerun.sh — Copy previous stage rootfs

if [ ! -d "${ROOTFS_DIR}" ]; then
	copy_previous
fi