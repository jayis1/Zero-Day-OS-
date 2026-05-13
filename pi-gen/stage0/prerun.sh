#!/bin/bash -e
# stage0/prerun.sh — Bootstrap the base Debian arm64 system
# Uses qemu-aarch64-static for cross-architecture debootstrap inside Docker
# Bypasses the bootstrap() function in scripts/common to avoid GPG keyring issues

if [ ! -d "${ROOTFS_DIR}" ]; then
	mkdir -p "${ROOTFS_DIR}"

	# Copy qemu-aarch64-static into the target rootfs so debootstrap can
	# execute arm64 binaries during the second (native) stage
	QEMU_BIN=""
	for candidate in /usr/bin/qemu-aarch64-static /usr/bin/qemu-aarch64; do
		if [ -x "${candidate}" ]; then
			QEMU_BIN="${candidate}"
			break
		fi
	done

	if [ -n "${QEMU_BIN}" ]; then
		mkdir -p "${ROOTFS_DIR}/usr/bin"
		cp "${QEMU_BIN}" "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
		chmod +x "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
		echo "[zeroday] Copied ${QEMU_BIN} into rootfs for arm64 emulation"
	else
		echo "[zeroday] WARNING: qemu-aarch64-static not found — debootstrap may fail"
	fi

	# For Bookworm: non-free-firmware is a separate component (Debian 12+)
	BOOTSTRAP_ARGS=(
		--arch arm64
		--no-check-gpg
		--components main,contrib,non-free,non-free-firmware
		--exclude=info,ifupdown
		--include=ca-certificates
	)

	debootstrap "${BOOTSTRAP_ARGS[@]}" "${RELEASE}" "${ROOTFS_DIR}" http://deb.debian.org/debian/ || {
		BOOTSTRAP_EXIT=$?
		rm -f wget-log*
		log "debootstrap failed with exit code ${BOOTSTRAP_EXIT}"
		false
	}

	rm -f wget-log*

	# Verify debootstrap succeeded
	if [ -d "${ROOTFS_DIR}/debootstrap" ] && ! rmdir "${ROOTFS_DIR}/debootstrap" 2>/dev/null; then
		cp "${ROOTFS_DIR}/debootstrap/debootstrap.log" "${STAGE_WORK_DIR}" 2>/dev/null || true
		log "bootstrap failed: please check ${STAGE_WORK_DIR}/debootstrap.log"
		false
	fi

	# Remove qemu binary from rootfs — not needed at runtime on real hardware
	rm -f "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
fi