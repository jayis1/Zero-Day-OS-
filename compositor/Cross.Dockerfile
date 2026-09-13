FROM debian:bookworm

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        gcc-aarch64-linux-gnu \
        g++-aarch64-linux-gnu \
        pkg-config \
        libwayland-dev:arm64 \
        libdrm-dev:arm64 \
        libgbm-dev:arm64 \
        libegl-dev:arm64 \
        libgles-dev:arm64 \
        libinput-dev:arm64 \
        libudev-dev:arm64 \
        libseat-dev:arm64 \
        libevdev-dev:arm64 \
        libxkbcommon-dev:arm64 \
        libpixman-1-dev:arm64 \
        libsystemd-dev:arm64 \
        wayland-protocols \
        crossbuild-essential-arm64 \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig
ENV PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig