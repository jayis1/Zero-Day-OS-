#!/bin/bash -e
# stage0/prerun.sh — Bootstrap the base Debian system
# Uses Debian armhf with --no-check-gpg (RPi packages added in stage1)
# Bypasses the bootstrap() function in scripts/common to avoid GPG keyring issues
# that occur inside Docker where RPi keys cannot be downloaded.

if [ ! -d "${ROOTFS_DIR}" ]; then
	mkdir -p "${ROOTFS_DIR}"

	# Build debootstrap arguments array
	# For Bookworm: non-free-firmware is a separate component (Debian 12+)
	BOOTSTRAP_ARGS=(
		--arch armhf
		--no-check-gpg
		--components main,contrib,non-free,non-free-firmware
		--exclude=info,ifupdown
		--include=ca-certificates
		"${RELEASE}"
		"${ROOTFS_DIR}"
		http://deb.debian.org/debian/
	)

	# Format args for capsh shell invocation
	printf -v BOOTSTRAP_STR '%q ' "${BOOTSTRAP_ARGS[@]}"

	# Run debootstrap inside 32-bit personality (armhf compat on amd64 host)
	if ! setarch linux32 capsh $CAPSH_ARG -- -c "debootstrap $BOOTSTRAP_STR"; then
		rm -f wget-log*
		log "debootstrap failed with exit code $?"
		false
	fi

	rm -f wget-log*

	# Verify debootstrap succeeded
	if [ -d "${ROOTFS_DIR}/debootstrap" ] && ! rmdir "${ROOTFS_DIR}/debootstrap" 2>/dev/null; then
		cp "${ROOTFS_DIR}/debootstrap/debootstrap.log" "${STAGE_WORK_DIR}" 2>/dev/null || true
		log "bootstrap failed: please check ${STAGE_WORK_DIR}/debootstrap.log"
		false
	fi
fi