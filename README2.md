# Building ZERO-DAY OS — Local Build Guide

This guide covers building the ZERO-DAY OS image on your own Linux machine.
GitHub Actions cannot do it in time — the arm64 QEMU build takes 3-4 hours.

---

## Requirements

**OS:** Linux x86_64 (Ubuntu 22.04+ or Debian 12+ recommended)
**RAM:** 8 GB minimum, 16 GB recommended
**Disk:** 40 GB free (rootfs + work dirs + final image + zip)
**Time:** 3-4 hours first run, ~1 hour with apt-cacher-ng cache

**Packages to install:**

    sudo apt-get update
    sudo apt-get install -y \
        docker.io qemu-user-static binfmt-support \
        git curl zip unzip

    sudo systemctl enable --now docker
    sudo usermod -aG docker $USER
    # log out and back in, or: newgrp docker

Verify arm64 emulation works before starting:

    docker run --rm --platform linux/arm64 arm64v8/alpine uname -m
    # must print: aarch64

---

## Get the repo

    git clone https://github.com/jayis1/Zero-Day-OS-
    cd Zero-Day-OS-

---

## Cross-compile Rust binaries first (optional but recommended)

The TUI dashboards (sys-tui, net-tui, loot-tui, scan-tui, trail-tui,
wifi-tui, bt-tui) and the file explorer (zeroday-fm) are Rust binaries
that need to be cross-compiled for arm64 before the image build.

Install cross:

    cargo install cross
    # or: cargo binstall cross

Build everything:

    # TUI dashboards
    cd tui
    cross build --release --target aarch64-unknown-linux-gnu
    cd ..

    # File explorer
    cd explorer
    cross build --release --target aarch64-unknown-linux-gnu
    cd ..

    # Compositor (Wayland/DRM compositor for 320x170 LCD)
    cd compositor
    cross build --release --target aarch64-unknown-linux-gnu
    cd ..

    # Terminal emulator
    cd terminal
    cross build --release --target aarch64-unknown-linux-gnu
    cd ..

    # Breadcrumb navigation daemon
    cd trail
    cross build --release --target aarch64-unknown-linux-gnu
    cd ..

The pi-gen stage install scripts look for pre-built binaries in
`<crate>/target/aarch64-unknown-linux-gnu/release/`. If they are missing,
those components are skipped with a WARNING — the image still builds,
just without those binaries.

---

## Speed up apt downloads with a local cache (optional but saves ~45 min)

    sudo apt-get install -y apt-cacher-ng
    sudo systemctl enable --now apt-cacher-ng

Then uncomment this line in `pi-gen/config`:

    APT_PROXY="http://172.17.0.1:3142/"

`172.17.0.1` is the Docker bridge gateway — visible from inside the build
container. The first build populates the cache; every run after is fast.

---

## Configure the build

The config file is already committed with sensible defaults.
Check `pi-gen/config` and adjust if needed:

    IMG_NAME="zeroday-os"      # output filename prefix
    RELEASE="bookworm"         # Debian/Raspbian release
    ARCH="arm64"               # target architecture
    FIRST_USER_PASS="zeroday"  # change this if you want

Skip stage5 (full desktop — not needed, saves ~30 min):

    touch pi-gen/stage5/SKIP pi-gen/stage5/SKIP_IMAGES

---

## Run the build

    cd pi-gen
    CONTINUE=0 ./build-docker.sh

`CONTINUE=0` starts fresh. If the build is interrupted you can resume:

    CONTINUE=1 ./build-docker.sh

The script will:
1. Build the Docker image (`pi-gen/Dockerfile`) — ~3 min
2. Launch a privileged container with QEMU arm64 registered
3. Run `build.sh` through all stages inside the container — 3-4 hours
4. Output images to `pi-gen/deploy/`

Watch progress (in another terminal):

    docker logs -f zeroday_pigen

---

## Output

    pi-gen/deploy/
      YYYY-MM-DD-zeroday-os-full.zip   (~1.1 GB)
      YYYY-MM-DD-zeroday-os-lite.zip   (~1.2 GB)
      YYYY-MM-DD-zeroday-os-full.img
      build.log

Flash to SD card:

    unzip YYYY-MM-DD-zeroday-os-full.zip
    sudo dd if=YYYY-MM-DD-zeroday-os-full.img of=/dev/sdX bs=4M status=progress conv=fsync

Or use Raspberry Pi Imager: File -> Use custom image -> select the .img file.

---

## Troubleshooting

**"qemu-aarch64-static not found"**

    sudo apt-get install -y qemu-user-static
    sudo update-binfmts --enable qemu-aarch64

**"Container zeroday_pigen already exists"**

    docker rm -v zeroday_pigen
    CONTINUE=0 ./build-docker.sh

**Build hangs at a package install**

A single apt package sometimes stalls under QEMU emulation. Check which one:

    docker logs zeroday_pigen | tail -20

Then add a `|| true` fallback to that stage's `01-run.sh` and resume:

    CONTINUE=1 ./build-docker.sh

**Out of disk space mid-build**

    df -h /
    # Free space, then:
    docker rm -v zeroday_pigen
    CONTINUE=0 ./build-docker.sh

**Build works but Rust binaries missing from image**

You skipped the cross-compile step. Run it (see above), then rebuild
from stage3 onward:

    # Skip stages 0-2 (already done) and rebuild from stage3
    touch pi-gen/stage0/SKIP pi-gen/stage1/SKIP pi-gen/stage2/SKIP
    CONTINUE=1 ./build-docker.sh

---

## Typical build timeline

    0:00  Docker image build (Dockerfile)
    0:03  Stage 0 — bootstrap (debootstrap arm64 under QEMU)  ~45 min
    0:48  Stage 1 — base system packages                       ~30 min
    1:18  Stage 2 — base config, users, ssh                    ~10 min
    1:28  Stage 3 — compositor, terminal, explorer, TUI        ~20 min
    1:48  Stage 4 — pentest tools (17 substages)               ~90 min
    3:18  Image creation, compression, zip                     ~15 min
    3:33  Done

Total: ~3.5 hours cold. ~1 hour with apt-cacher-ng cache warm.
