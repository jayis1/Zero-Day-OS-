ARG BASE_IMAGE=debian:trixie
FROM ${BASE_IMAGE}

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get -y update && \
    apt-get -y install --no-install-recommends \
        git vim parted \
        quilt coreutils qemu-user-static debootstrap zerofree zip dosfstools e2fsprogs \
        libarchive-tools libcap2-bin rsync grep udev xz-utils curl xxd file kmod bc \
        binfmt-support ca-certificates fdisk gpg pigz arch-test \
        debian-archive-keyring \
    && rm -rf /var/lib/apt/lists/*

# Download RPi GPG keys — try multiple sources
# These are needed for debootstrap to verify Raspbian packages
RUN mkdir -p /usr/share/keyrings && \
    curl -sL https://www.raspberrypi.com/raspberrypi.gpg.key 2>/dev/null | gpg --dearmor -o /usr/share/keyrings/raspberrypi-archive-keyring.gpg 2>/dev/null; \
    curl -sL https://archive.raspberrypi.org/debian/raspberrypi.gpg.key 2>/dev/null | gpg --dearmor -o /usr/share/keyrings/raspbian-archive-keyring.gpg 2>/dev/null; \
    curl -sL https://raspbian.raspberrypi.org/raspbian.public.key 2>/dev/null | gpg --dearmor -o /usr/share/keyrings/raspbian-archive-keyring.gpg 2>/dev/null; \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 82B129927FA3303E 2>/dev/null && \
    apt-key export 82B129927FA3303E 2>/dev/null | gpg --dearmor -o /usr/share/keyrings/raspbian-archive-keyring.gpg 2>/dev/null; \
    echo "GPG key retrieval complete (some may have failed)"

# Ensure at least an empty keyring file exists so debootstrap doesn't crash
RUN touch /usr/share/keyrings/raspberrypi-archive-keyring.gpg && \
    touch /usr/share/keyrings/raspbian-archive-keyring.gpg && \
    mkdir -p /pi-gen/stage0/files && \
    cp /usr/share/keyrings/raspberrypi-archive-keyring.gpg /pi-gen/stage0/files/raspberrypi.gpg 2>/dev/null || true

COPY . /pi-gen/

VOLUME [ "/pi-gen/work", "/pi-gen/deploy" ]