```
 ███████╗███████╗██████╗ ███████╗    ██████╗ ███████╗██████╗ ██╗   ██╗███████╗████████╗███████╗██████╗ 
 ██╔════╝██╔════╝██╔══██╗██╔════╝    ██╔══██╗██╔════╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔════╝██╔══██╗
 █████╗  █████╗  ██████╔╝███████╗    ██████╔╝█████╗  ██████╔╝██║   ██║███████╗   ██║   █████╗  ██████╔╝
 ██╔══╝  ██╔══╝  ██╔══██╗╚════██║    ██╔══██╗██╔══╝  ██╔═══╝ ██║   ██║╚════██║   ██║   ██╔══╝  ██╔══██╗
 ██║     ███████╗██║  ██║███████║    ██║  ██║███████╗██║     ╚██████╔╝███████║   ██║   ███████╗██║  ██║
 ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝    ╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝ ╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝
                                         ╔════════════════════════════════════════╗
                                         ║  PRE-RELEASE TECHNICAL BLUEPRINT      ║
                                         ║  Build Architecture & Implementation   ║
                                         ╚════════════════════════════════════════╝
```

# ZERO-DAY OS — Technical Build Blueprint

**Target Hardware:** M5Stack Cardputer Zero (Raspberry Pi CM0, BCM2837, Quad-Core Cortex-A53 @ 1GHz, 512MB LPDDR2)  
**Form Factor:** 85x54mm credit-card, built-in 46-key keyboard, 1.9" ST7789v3 LCD, 1500mAh battery  
**Build System:** Fork of `pi-gen` (official Raspberry Pi OS image builder) with custom stages  
**Base OS:** Debian 12 (bookworm) minimal + Kali Rolling repository overlay  
**Status:** PRE-RELEASE — hardware not yet shipped, GPIO pinout pending final DTS

---

## 1. Project Structure

```
zeroday-os/
├── pi-gen/                          # Forked pi-gen with custom stages
│   ├── config                        # Build configuration
│   │   ├── rpi-cm0                   # CM0-specific config
│   │   └── common                    # Shared config
│   ├── stage0/                       # Bootstrap (debootstrap)
│   ├── stage1/                       # Base system
│   ├── stage2/                       # Networking + core utils
│   ├── stage3/                       # ZERO-DAY OS core (our Stage A)
│   │   ├── 00-configure-base         #   System tuning, users, locale
│   │   ├── 01-kernel-dtb             #   Custom kernel + device tree
│   │   ├── 02-xorg-i3               #   Minimal X + i3 WM
│   │   ├── 03-boot-scripts           #   Auto-start, panic, power mgmt
│   │   ├── 04-hardware-enable        #   LCD, keyboard, audio, IMU, RTC
│   │   ├── 05-terminal-st            #   st terminal, fbterm fallback
│   │   └── 06-flipper-tui            #   Pygame GUI launcher
│   ├── stage4/                       # Hacking tools (our Stage B)
│   │   ├── 00-kali-repos             #   Add Kali rolling repos
│   │   ├── 01-wifi-tools             #   aircrack-ng, hcxdumptool, hostapd
│   │   ├── 02-network-tools          #   nmap, gobuster, dsniff, responder, chisel
│   │   ├── 03-bluetooth-tools        #   bluez, bettercap
│   │   ├── 04-exploit-tools          #   sqlmap, exploitdb, john, hydra, strace
│   │   ├── 05-reverse-shell-kit      #   netcat, socat, ncat
│   │   ├── 06-ir-tools               #   lirc, ir-utils
│   │   ├── 07-camera-tools           #   libcamera, fswebcam, tesseract
│   │   ├── 08-sdr-tools              #   rtl-433 (rtl-sdr dependent)
│   │   ├── 09-wordlists-seclists     #   SecLists (compressed)
│   │   ├── 10-wifi-dongle            #   RTL8821CU DKMS driver
│   │   ├── 11-subghz-nfc-tools       #   Sub-GHz CC1101 + NFC PN532 tools
│   │   ├── 12-meshtastic-tools       #   Meshtastic LoRa mesh
│   │   └── 13-media-tools             #   ffplay, alsa-utils (radio + walkie-talkie)
│   ├── stage5/                       # Zero-touch setup (our Stage C)
│   │   ├── 00-first-boot             #   First-boot wizard
│   │   ├── 01-opencode               #   OpenCode CLI install
│   │   ├── 02-opencode-session       #   tmux IDE wrapper script
│   │   └── 03-cleanup                #   Trim, compress, finalize
│   ├── deploy/                       # Output: .img files
│   └── Dockerfile                    # Reproducible build container
│
├── overlays/                         # Device Tree Overlays
│   ├── st7789v3-overlay.dts          #   LCD display
│   ├── cardputer-kbd-overlay.dts     #   46-key matrix keyboard
│   ├── es8389-overlay.dts            #   Audio codec (I2S)
│   ├── imx219-overlay.dts            #   Camera (CSI)
│   ├── bmi270-overlay.dts            #   IMU (I2C)
│   ├── rx8130ce-overlay.dts          #   RTC (I2C)
│   ├── bq27220-overlay.dts           #   Battery fuel gauge (I2C)
│   └── ir-trx-overlay.dts            #   IR transceiver (GPIO)
│
├── scripts/                          # One-key hacking scripts
│   ├── wifi/
│   │   ├── wifi-scan
│   │   ├── wifi-deauth
│   │   ├── wifi-handshake
│   │   ├── wifi-pmkid
│   │   ├── wifi-evil-twin          # Rogue AP + captive portal
│   │   ├── wifi-crack
│   │   ├── wifi-monitor-toggle
│   │   └── wifi-survey-log           # Continuous WiFi survey logger
│   ├── network/
│   │   ├── net-discover
│   │   ├── net-quickscan
│   │   ├── net-vulnscan
│   │   ├── quick-c2                 # Encrypted C2 listener (socat + OpenSSL)
│   │   ├── doh-proxy                # DNS-over-HTTPS proxy
│   │   ├── tunnel-mgr               # SSH tunnel manager (SOCKS/forward/reverse)
│   │   ├── iot-scan                 # IoT-focused Nmap scan presets
│   │   ├── ragnar-scan               # Autonomous 3-phase network recon
│   │   ├── threat-intel               # CVE/CISA KEV lookup
│   │   └── device-classify            # Device fingerprinting from nmap
│   ├── bluetooth/
│   │   ├── bt-scan
│   │   ├── bt-deep
│   │   ├── bt-attack
│   │   └── ble-gatt
│   ├── reverse/
│   │   ├── revshell-listen
│   │   ├── revshell-gen
│   │   └── revshell-stabilize
│   ├── ir/
│   │   ├── ir-scan
│   │   ├── ir-replay
│   │   └── ir-brute
│   ├── camera/
│   │   ├── cam-snap
│   │   ├── cam-stream
│   │   └── cam-ocr
│   ├── subghz/
│   │   ├── subghz-scan            # CC1101 frequency scanner
│   │   ├── subghz-record           # Sub-GHz signal recorder
│   │   └── subghz-replay           # Sub-GHz signal replay
│   ├── nfc/
│   │   ├── nfc-read                # NFC/RFID tag reader
│   │   ├── nfc-clone               # NFC tag cloner
│   │   └── nfc-emulate             # NFC tag emulator
│   ├── mesh/
│   │   ├── mesh-chat               # Meshtastic mesh messaging
│   │   └── mesh-setup              # Meshtastic install/config/relay
│   ├── dongle/
│   │   └── dongle-setup            # RTL8821CU manager
│   ├── hardware/
│   │   ├── sdr-scan
│   │   ├── gpio-probe
│   │   ├── rf-capture
│   │   ├── cardputer-battery
│   │   ├── monsterctl
│   │   └── install-janos             # JanOS-app installer/launcher
│   ├── system/
│   │   ├── panic
│   │   ├── zeroday-bootanim        # Boot animation (glitch ASCII)
│   │   ├── cardputer-wifi-setup
│   │   ├── cardputer-wifi-toggle
│   │   ├── power-mode
│   │   ├── tamper-watch
│   │   ├── usb-gadget-mode
│   │   ├── mac-rotate               # On-demand MAC randomization
│   │   ├── loot-organize            # On-demand loot directory organizer
│   │   ├── opencode-session
│   │   └── opencode-ask
│   └── tui/
│       └── cyber_launcher.py         #   Pygame GUI launcher (320x170)
│
├── configs/                          # System configs
│   ├── i3/
│   │   └── config                    #   i3 keybindings + Omni-Key
│   ├── st/
│   │   └── config.h                  #   st terminal config (small screen)
│   ├── systemd/
│   │   ├── zeroday-boot.service      #   Boot orchestration
│   │   ├── panic.service             #   Emergency kill service
│   │   ├── tamper-watch.service      #   IMU tamper detection
│   │   ├── power-governor.service    #   CPU frequency scaling
│   │   └── opencode.service         #   OpenCode launch service
│   ├── fbterm/
│   │   └── fbterm.conf               #   Framebuffer terminal config
│   ├── xorg/
│   │   └── xorg.conf                 #   Minimal X config for ST7789
│   ├── bash/
│   │   └── .bashrc                   #   Custom PS1, aliases, PATH
│   └── motd/
│       └── motd                       #   Login banner (ASCII)
│
└── README.md                          # You are here
```

---

## 2. Boot Sequence

```
┌────────────────────────────────────────────────────────────────┐
│                    ZERO-DAY OS BOOT SEQUENCE                    │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  [0.0s] Power On                                               │
│         │                                                       │
│  [0.1s] BCM2837 Boot ROM                                      │
│         │  bootcode.bin → start.elf → firmware                  │
│         │  /boot/config.txt loaded                             │
│         │  Device Tree + Overlays applied                      │
│         │                                                       │
│  [1.5s] Linux Kernel (bcm2837)                                │
│         │  initramfs → switch_root                             │
│         │  /boot/config.txt:                                    │
│         │    dtoverlay=st7789v3                                │
│         │    dtoverlay=cardputer-kbd                           │
│         │    dtoverlay=es8389                                  │
│         │    dtoverlay=imx219                                  │
│         │    dtoverlay=bmi270                                  │
│         │    dtoverlay=rx8130ce                               │
│         │    dtoverlay=bq27220                                 │
│         │    dtoverlay=ir-trx                                  │
│         │    gpu_mem=16                                        │
│         │    disable_camera_led=1                             │
│         │    hdmi_force_hotplug=0                             │
│         │    max_framebuffers=1                               │
│         │                                                       │
│  [3.0s] systemd (PID 1)                                       │
│         │  ┌─────────────────────────────────────┐             │
│         │  │  zeroday-boot.service               │             │
│         │  │  ├─ Set CPU governor (performance)  │             │
│         │  │  ├─ Load BQ27220 battery module     │             │
│         │  │  ├─ Set framebuffer to ST7789        │             │
│         │  │  ├─ Play boot animation             │             │
│         │  │  │  (zeroday-bootanim — glitch ASCII)│            │
│         │  │  ├─ Auto-login root on tty1         │             │
│         │  │  └─ Start i3 via startx             │             │
│         │  └─────────────────────────────────────┘             │
│         │                                                       │
│  [5.0s] Xorg + i3                                             │
│         │  st terminal auto-starts                             │
│         │  tmux default session created                        │
│         │                                                       │
│  [6.0s] cyber_launcher GUI (Pygame)                          │
│         │  Auto-launches full-screen in st                     │
│         │  Binds to the ST7789 framebuffer                    │
│         │  2x5 category grid renders on 1.9"                  │
│         │                                                       │
│  [7.0s] READY                                                  │
│         │  WiFi: down (stealth by default)                     │
│         │  BT: down (stealth by default)                       │
│         │  Radios activated only on user command               │
│         │  Battery check: cardputer-battery                    │
│         │                                                       │
└────────────────────────────────────────────────────────────────┘
```

**Key boot decisions:**
- **No display manager.** Auto-login `root` on `tty1`, `.bash_profile` launches `startx` with i3.
- **Radios OFF at boot.** WiFi and Bluetooth are disabled by default. Activated only when you need them. Zero RF signature on power-up.
- **GPU mem capped at 16MB.** We're running a TUI, not a desktop. The other 496MB belongs to userland.
- **HDMI disabled by default.** `hdmi_force_hotplug=0`. The 1.9" LCD is the primary display. HDMI can be enabled via `cardputer-hdmi on`.

---

## 3. Memory Budget (512MB Total)

Every megabyte is accounted for:

| Component | RAM | Notes |
|---|---|---|
| **Linux Kernel** | ~12 MB | Stripped config, no unused modules |
| **systemd** | ~15 MB | Minimal units, no NetworkManager |
| **Xorg** | ~20 MB | fbdev driver, no GPU acceleration |
| **i3 WM** | ~5 MB | Tiling only, no decorations |
| **st terminal** | ~3 MB | Single instance, tmux handles splits |
| **tmux** | ~4 MB | Session management |
| **Pygame GUI** | ~25 MB | Python + pygame + SDL2 |
| **Bash + core utils** | ~10 MB | Busybox where possible |
| **dropbear (SSH)** | ~2 MB | On-demand only, not running at boot |
| **wpa_supplicant** | ~8 MB | Started on-demand only |
| **bluetoothd** | ~10 MB | Started on-demand only |
| **Reserved GPU** | ~16 MB | Capped via `gpu_mem=16` |
| **TOTAL SYSTEM** | **~130 MB** | |
| **FREE FOR TOOLS** | **~382 MB** | Enough for nmap, aircrack, hashcat |

When a heavy tool runs (like Metasploit), the TUI can be backgrounded to free ~25MB. The `power-mode stealth` profile drops to single core and kills Xorg entirely (switching to fbterm direct), saving another ~25MB.

---

## 4. Stage Layout (pi-gen Fork)

### Stage 0 — Bootstrap (upstream pi-gen)
Standard `debootstrap` Debian bookworm. No modifications.

### Stage 1 — Base System (upstream pi-gen)
Kernel, boot firmware, base packages. No modifications.

### Stage 2 — Networking (upstream pi-gen)
Network stack, DHCP, filesystem tools. No modifications.

### Stage 3 — ZERO-DAY OS Core (Custom)

#### 03-stage/00-configure-base
```bash
# System identity
echo "zeroday" > /etc/hostname
echo "127.0.1.1  zeroday" >> /etc/hosts

# User setup
echo "root:zeroday" | chpasswd
useradd -m -s /bin/bash operator
echo "operator:zeroday" | chpasswd
echo "operator ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/operator

# Locale + timezone
dpkg-reconfigure -f noninteractive locales
echo "UTC" > /etc/timezone
ln -sf /usr/share/zoneinfo/UTC /etc/localtime

# Swappiness — minimize SD card writes
echo "vm.swappiness=1" >> /etc/sysctl.d/99-zeroday.conf
echo "vm.dirty_ratio=10" >> /etc/sysctl.d/99-zeroday.conf
echo "vm.dirty_background_ratio=5" >> /etc/sysctl.d/99-zeroday.conf

# Reduce SD card writes — log to RAM
echo "tmpfs /tmp tmpfs defaults,noatime,nosuid,size=64m 0 0" >> /etc/fstab
echo "tmpfs /var/log tmpfs defaults,noatime,nosuid,size=16m 0 0" >> /etc/fstab
echo "tmpfs /var/tmp tmpfs defaults,noatime,nosuid,size=16m 0 0" >> /etc/fstab

# Network — systemd-networkd, no NetworkManager
# WiFi and BT are DOWN at boot (stealth default)
# rfkill blocks all wireless on boot

# Disable unnecessary services
systemctl disable bluetooth
systemctl disable ModemManager
systemctl disable avahi-daemon
systemctl disable cups
systemctl disable triggerhappy
```

#### 03-stage/01-kernel-dtb
```bash
# Compile and install device tree overlays
# Uses dtc (device tree compiler) to build .dtbo from .dts sources
# Outputs to /boot/overlays/

# /boot/config.txt additions:
# dtoverlay=st7789v3          # LCD
# dtoverlay=cardputer-kbd     # Keyboard matrix
# dtoverlay=es8389             # Audio codec
# dtoverlay=imx219             # Camera
# dtoverlay=bmi270             # IMU
# dtoverlay=rx8130ce           # RTC
# dtoverlay=bq27220            # Battery fuel gauge
# dtoverlay=ir-trx             # IR transceiver

# Kernel config: strip unused modules
# Build as module: WiFi (brcmfmac), BT (hciattach), audio (snd_bcm2835)
# Build in: SPI, I2C, I2S, CSI, GPIO, framebuffer
```

#### 03-stage/02-xorg-i3
```bash
# Install minimal X + i3
apt install -y --no-install-recommends \
    xserver-xorg-core \
    xserver-xorg-video-fbdev \
    xserver-xorg-input-libinput \
    i3-wm \
    i3status \
    stterm \
    xdotool

# Configure Xorg for ST7789v3
# /etc/X11/xorg.conf:
#   Section "Device"
#     Identifier "ST7789"
#     Driver "fbdev"
#     Option "fbdev" "/dev/fb0"
#   EndSection
#   Section "Screen"
#     Identifier "LCD"
#     Device "ST7789"
#     DefaultDepth 16
#   EndSection

# i3 config (see configs/i3/config)
# - No title bars, no borders, no gaps
# - Single window fullscreen by default
# - All interactions via Fn key combos
# - $mod set to Mod1 (Alt) which Fn maps to
```

#### 03-stage/03-boot-scripts
```bash
# /etc/systemd/system/zeroday-boot.service
# After=multi-user.target
# ExecStart=/usr/local/bin/zeroday-boot
#
# zeroday-boot:
#   1. Set CPU governor to 'performance' or 'ondemand'
#   2. Load BQ27220 battery module
#   3. rfkill block all wireless (stealth default)
#   4. Set LCD brightness via sysfs
#   5. Auto-login root on tty1

# /etc/systemd/system/panic.service
# Emergency kill + wipe service
# Triggered by Fn+P keybinding or 'panic' command
#
# panic:
#   1. kill -9 (all offensive processes)
#   2. rm -f ~/.bash_history /tmp/* /opt/cardputer/loot/*
#   3. history -c && history -w
#   4. tmux kill-server
#   5. clear && echo "ZERODAY OS v0.1 — Login:" 
#   6. rfkill block all wireless
#   7. echo "$(date) PANIC" >> /opt/cardputer/panic.log

# /etc/systemd/system/tamper-watch.service
# Runs tamper-watch daemon:
#   - Reads BMI270 IMU via sysfs/i2c
#   - If movement exceeds threshold while in stealth:
#     - Lock terminal (vlock)
#     - If violent movement: wipe /opt/cardputer/loot/ and .bash_history
#   - Configurable via /opt/cardputer/config/tamper.conf
```

#### 03-stage/04-hardware-enable
```bash
# ST7789v3 LCD
# - Framebuffer device at /dev/fb0
# - Rotation: landscape for keyboard orientation
# - DPI scaling for 1.9" screen (fonts 8-10pt maximum)

# 46-key matrix keyboard
# - Uses i2c-hid or custom GPIO scanner
# - Keymap: standard US layout with Fn layer
# - Fn acts as Alt (Mod1) for i3 keybindings
# - /etc/keyboard-layout: custom xkb or setxkbmap variant

# ES8389 audio codec
# - I2S driver, ALSA config
# - MEMS mic: default capture device
# - 1W speaker: default output (with limiter to prevent damage)
# - 3.5mm TRS: auto-switch when plugged

# IMX219 camera
# - libcamera-based (not legacy bcm2835-v4l2)
# - v4l2 device at /dev/video0
# - Maximum 1080p30 H.264 encode via BCM video codec

# BQ27220 fuel gauge
# - /sys/class/power_supply/bq27220/
# -电压 (voltage), capacity (%), status (charging/discharging)
# - Power display in i3status bar

# BMI270 IMU
# - /sys/bus/iio/devices/iio:device0/
# - Accelerometer + gyroscope readings
# - tamper-watch daemon reads this

# RX8130CE RTC
# - /dev/rtc0
# - hwclock --systohc at shutdown
# - hwclock --hctosys at boot

# IR transceiver
# - /dev/lirc0 (lirc kernel module)
# - TX: send raw IR signals via lirc
# - RX: capture IR signals via lirc
# - Also: raw GPIO bitbanging fallback

# Ethernet (LAN8720 / USB-LAN)
# - 10/100M via systemd-networkd
# - DHCP client by default
# - Can be set to static via cardputer-wifi-setup

# USB-C Device Mode (gadget)
# - ConfigFS USB gadget
# - usb-gadget-mode script toggles between:
#   - hid: USB keyboard (Rubber Ducky mode)
#   - serial: USB serial console (debug)
#   - ncm: USB network (for Mac/Linux host networking)
#   - mass_storage: USB drive (exfil mode)
```

#### 03-stage/05-terminal-st
```bash
# st (simple terminal) — compiled from source
# Custom config:
#   - Font: Terminus 8pt (readable on 1.9" screen)
#   - No scrollback (tmux handles that)
#   - No true color (256 color for performance)
#   - Dimensions: hardcoded for 320x170 framebuffer
#   - Selection via keyboard only (no mouse)

# fbterm as fallback (no X11 needed)
# Used when Xorg is killed in stealth power mode
# /etc/fbterm.conf:
#   font-size=8
#   color-mode=256
```

#### 03-stage/06-flipper-tui
```bash
# cyber_launcher — Python Pygame application
# /usr/local/bin/cyber_launcher
#
# Architecture:
#   App (Pygame)
#   ├── State: HOME (Level 1 — 4x3 icon grid)
#   │   ├── [WIFI]  → List → Action/Prompt
#   │   ├── [M5MONSTER] → List → Action/Prompt
#   │   ├── [NET]   → List → Action/Prompt
#   │   ├── [BT]    → List → Action/Prompt
#   │   ├── [IR]    → List → Action/Prompt
#   │   ├── [CAM]   → List → Action/Prompt
#   │   ├── [PAYLD] → List → Action/Prompt
#   │   ├── [RADIO] → List → WALKIE_TALKIE inline
#   │   ├── [MEDIA] → List → MEDIA_PLAYER inline
#   │   ├── [SHELL] → List → Action/Prompt
#   │   ├── [SYS]   → List → Action/Prompt
#   │   └── [OPEN]  → List → Action/Prompt
#   │
#   ├── State: LIST (Level 2 — scrollable tool list)
#   │   └── Each category has its tools listed with descriptions
#   │
#   └── State: PROMPT (Level 3 — argument input for commands)
#       ├── Input validation with regex per argument type
#       ├── shlex.quote() for all arguments
#       ├── [Enter] to execute (spawns st terminal)
#       └── [Tab] to cycle between argument fields
#
# Inline states (no terminal spawn):
#   WALKIE_TALKIE: UDP broadcast PTT, port 42420, 30s timeout
#   MEDIA_PLAYER:  ffplay-based radio + local music (shuffle)
#
# Key bindings:
#   ↑↓←→   Navigate
#   Enter   Drill down / Execute
#   Esc     Go back / Exit
#   Space   PTT (Walkie Talkie mode only)
#
# The GUI renders via Pygame (SDL2) on DRM/KMS or X11
```

### Stage 4 — Hacking Tools (Custom)

#### 04-stage/00-kali-repos
```bash
# Add Kali Rolling repository with low priority
# /etc/apt/sources.list.d/kali.list:
#   deb http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware
#
# /etc/apt/preferences.d/kali.pref:
#   Package: *
#   Pin: release o=Kali
#   Pin-Priority: 50
#
# This means: Kali packages are ONLY installed when explicitly requested
# They will never auto-upgrade Debian packages

apt update
```

#### 04-stage/01-wifi-tools
```bash
apt install -y --no-install-recommends \
    aircrack-ng \
    hcxdumptool \
    hcxtools \
    hostapd \
    dnsmasq \
    iw \
    rfkill \
    wpasupplicant

# Install wifi-scan, wifi-deauth, wifi-handshake, wifi-pmkid,
# wifi-evil-twin, wifi-crack, wifi-monitor-toggle
# to /usr/local/bin/
```

#### 04-stage/02-network-tools
```bash
apt install -y --no-install-recommends \
    nmap \
    netcat-openbsd \
    socat \
    tcpdump \
    dnsutils \
    ethtool \
    python3-scapy \
    python3-serial \
    jq \
    dsniff \
    gobuster

# Install net-discover, net-quickscan, net-vulnscan,
# quick-c2, doh-proxy, tunnel-mgr, iot-scan
# to /usr/local/bin/

# Responder — LLMNR/NBT-NS poisoner (Kali, best-effort)
apt -t kali-rolling install -y --no-install-recommends responder
```

#### 04-stage/03-bluetooth-tools
```bash
apt install -y --no-install-recommends \
    bluez \
    bluez-hid2hci

# Bettercap — MITM framework (Kali, best-effort)
# Install bt-scan, bt-deep, bt-attack, ble-gatt
# to /usr/local/bin/
apt -t kali-rolling install -y --no-install-recommends bettercap
```

#### 04-stage/04-exploit-tools
```bash
apt install -y --no-install-recommends \
    strace \
    john \
    hydra

apt install -y --no-install-recommends sqlmap

# NOTE: metasploit-framework is NOT installed — requires 1GB+ RAM,
#       armhf has only 512MB. msfconsole OOM-kills within minutes.
#       Use quick-c2 for C2 listeners and payload generation instead.

apt -t kali-rolling install -y --no-install-recommends \
    exploitdb \
    hashcat-utils

# searchsploit is provided by exploitdb package
```

#### 04-stage/05-reverse-shell-kit
```bash
apt install -y --no-install-recommends \
    netcat-openbsd \
    socat \
    ncat

# Install revshell-listen, revshell-gen, revshell-stabilize
# to /usr/local/bin/
# These are bash scripts that generate/manipulate reverse shell one-liners
```

#### 04-stage/06-ir-tools
```bash
apt install -y --no-install-recommends \
    lirc \
    ir-keytable \
    mode2 \
    rc-core

# Install ir-scan, ir-replay, ir-brute
# to /usr/local/bin/
# ir-scan: captures raw IR signals via mode2, saves to /opt/cardputer/loot/ir/
# ir-replay: replays saved signals via lirc or raw GPIO
# ir-brute: iterates through known IR power codes for common devices
```

#### 04-stage/07-camera-tools
```bash
apt install -y --no-install-recommends \
    libcamera-tools \
    v4l-utils \
    fswebcam \
    tesseract-ocr \
    ffmpeg

# Install cam-snap, cam-stream, cam-ocr
# to /usr/local/bin/
# cam-snap:  libcamera-still → /opt/cardputer/loot/cam/
# cam-stream: libcamera-vid → H.264 .mp4 file
# cam-ocr:   cam-snap → tesseract → stdout + text file
```

#### 04-stage/08-sdr-tools
```bash
apt install -y --no-install-recommends \
    rtl-433

# rtl-sdr and python3-numpy may be pulled in as dependencies
# hackrf and soapysdr-tools are NOT included (too large for 512MB armhf)
# sdr-scan and rf-capture scripts use rtl_power / rtl_sdr when hardware is attached
```

#### 04-stage/09-wordlists-seclists
```bash
apt install -y --no-install-recommends -t kali-rolling \
    seclists

# Post-install: compress large wordlists to save SD card space
# Keep only: rockyou.txt, common-passwords, web-content, fuzzing
# Delete: deprecated, unused directories
# Result: ~200MB of wordlists instead of ~1GB
```

#### 04-stage/10-wifi-dongle
```bash
# RTL8821CU USB WiFi dongle driver (DKMS)
# Provides wlan1 for dual-radio attacks while wlan0 stays in managed mode

apt install -y --no-install-recommends \
    linux-headers-generic \
    dkms \
    rfkill

# Build 8821cu driver via DKMS on first boot (needs running kernel)
# dongle-setup script: install, status, monitor, managed, scan, test
# Udev rule: 70-persistent-net.rules ensures dongle always = wlan1
```

#### 04-stage/11-subghz-nfc-tools
```bash
# CC1101 Sub-GHz transceiver tools (SPI, 2.54mm 14-pin ExtPort)
# PN532 NFC/RFID module tools (I2C, Grove HY2.0-4P port)

# Sub-GHz: subghz-scan, subghz-record, subghz-replay
# NFC: nfc-read, nfc-clone, nfc-emulate

apt install -y --no-install-recommends \
    python3-pip \
    python3-spidev \
    python3-smbus2 \
    libnfc-dev

# CC1101 driver via spidev + custom GPIO (PLACEHOLDER — pin assignments pending hardware)
# PN532 via I2C (Grove port switches 1+2 ON for I2C mode)
```

#### 04-stage/12-meshtastic-tools
```bash
# Meshtastic LoRa mesh networking (UART, Grove HY2.0-4P port)
# PN532 NFC and Meshtastic LoRa share the Grove port — cannot be used simultaneously

# mesh-chat, mesh-setup scripts
# Meshtastic CLI installed on first boot (needs running Python/pip)

apt install -y --no-install-recommends \
    python3-venv \
    python3-pip

# Meshtastic CLI: pip3 install --break-system-packages meshtastic (on first boot or mesh-setup install)
# Wiring: Grove pin 1=VCC, pin 2=TXD, pin 3=RXD, pin 4=GND
```

#### 04-stage/13-media-tools
```bash
# Media playback — ffplay (from ffmpeg) for radio streaming and local music
# Wi-Fi Walkie-Talkie uses alsa-utils (arecord/aplay) for audio I/O

apt install -y --no-install-recommends \
    ffmpeg \
    alsa-utils

# webradio-danish — Danish web radio via ffplay (DR P1, DR P3, NOVA, POPFM)
# music-player — Local music player via ffplay (shuffle from /opt/cardputer/music)
```

### Stage 5 — Zero-Touch Setup (Custom)

#### 05-stage/00-first-boot
```bash
# /etc/systemd/system/first-boot.service
# Runs ONCE on first boot
#
# 1. Expand filesystem to fill SD card
# 2. Generate SSH host keys (dropbear)
# 3. Set random password for root (displayed on screen)
# 4. Prompt for operator password change
# 5. Configure WiFi (optional, can skip for stealth)
# 6. Write /etc/zeroday-release with build info
# 7. Disable first-boot.service (one-shot)

cat > /etc/zeroday-release << 'EOF'
ZERO-DAY OS v0.1-pre
Build: $(date +%Y%m%d)
Kernel: $(uname -r)
Hardware: M5Stack Cardputer Zero (CM0)
EOF
```

#### 05-stage/01-opencode
```bash
# Download and install OpenCode binary for ARM64
# OpenCode is an AI-assisted terminal-based code editor
# Installed to /usr/local/bin/opencode
#
# Configuration:
#   Config dir: /opt/cardputer/config/opencode/
#   Working dir: /opt/cardputer/workspace/
#   Model: configured at first run (local or API)
#
# Dependencies (already in base):
#   - tmux (for opencode-session)
#   - nano (fallback editor)
#   - python3 (if OpenCode needs it)
#   - git (version control within workspace)
```

#### 05-stage/02-opencode-session
```bash
# /usr/local/bin/opencode-session
# Wrapper script that:
#   1. Creates tmux session named "opencode"
#   2. Splits vertically: 70% top (OpenCode), 30% bottom (bash)
#   3. Sets working directory to /opt/cardputer/workspace/
#   4. If args given, opens specific file/dir in OpenCode
#
# Usage:
#   opencode-session                  # Full workspace
#   opencode-session /path/to/dir     # Specific directory
#   opencode-session /path file       # Specific file
```

#### 05-stage/03-cleanup
```bash
# Final image optimization
apt clean
apt autoremove --purge -y
rm -rf /var/cache/apt/archives/*
rm -rf /usr/share/doc/*
rm -rf /usr/share/man/*
rm -rf /usr/share/locale/*
find / -name "*.pyc" -delete
find / -name "__pycache__" -type d -exec rm -rf {} +

# Set filesystem to read-only on boot (optional, for SD card longevity)
# /etc/fstab: root=ro errors=remount-ro
# Can be toggled via 'cardputer-remount-rw'

# Disable swap (no swap partition, no swap file — we're RAM-only)
swapoff -a
rm -f /var/swap

# Final disk usage target: <3.5GB (fits on 4GB microSD with room to breathe)
# Recommended microSD: 32GB+ (for wordlists, captures, workspace)
```

---

## 5. i3 Window Manager Configuration

The entire user interaction model runs through i3 keybindings mapped to the `Fn` key:

```
# /etc/i3/config — ZERO-DAY OS

# ─── Core ───
set $mod Mod1                          # Fn maps to Alt
font pango:Terminus 8                  # Tiny font for 1.9" screen
default_border none                    # No borders
default_floating_border none           # No floating borders
hide_edge_borders both                 # No edge borders
focus_follows_mouse no                 # Focus by keyboard only

# ─── Startup ───
exec_always --no-startup-id feh --bg-scale /opt/cardputer/bg.png
exec_always --no-startup-id xset s off # No screensaver
exec_always --no-startup-id xset -dpms # No DPMS (we handle our own backlight)

# Auto-start the TUI
exec_always --no-startup-id st -e cyber_launcher

# ─── System Keybindings (Fn + Key) ───
bindsym $mod+Tab    exec --no-startup-id cyber_launcher      # Toggle TUI
bindsym $mod+p      exec --no-startup-id panic               # PANIC
bindsym $mod+space  exec --no-startup-id stealth-mode         # Kill backlight
bindsym $mod+Return exec --no-startup-id st -e tmux           # Quick terminal
bindsym $mod+q      kill                                      # Close window
bindsym $mod+o      exec --no-startup-id opencode-session     # OpenCode

# ─── Quick-Launch Keybindings ───
bindsym $mod+n      exec --no-startup-id st -e "sudo net-quickscan"
bindsym $mod+b      exec --no-startup-id st -e "sudo bt-scan"
bindsym $mod+s      exec --no-startup-id st -e "revshell-listen"
bindsym $mod+w      exec --no-startup-id cardputer-wifi-toggle
bindsym $mod+c      exec --no-startup-id cam-snap
bindsym $mod+i      exec --no-startup-id st -e "sudo ir-scan"
bindsym $mod+a      exec --no-startup-id opencode-ask

# ─── Tmux Integration ───
# All tool launches use 'st -e <command>' which opens in a new st window
# i3 tiles these as fullscreen tabs — Mod+1/2/3 to switch between them
# Or we use a single st with tmux inside it

# ─── Power Management Bar ───
bar {
    mode dock
    position bottom
    height 12
    font pango:Terminus 6
    status_command i3status
    colors {
        background #000000
        statusline #00ff00
    }
}

# i3status config:
#   - Battery % (BQ27220)
#   - CPU governor mode (perf/balanced/stealth)
#   - WiFi status (up/down/monitor)
#   - IP address (eth0/wlan0)
#   - Time (RTC)
```

---

## 6. Panic System — Technical Design

```bash
#!/bin/bash
# /usr/local/bin/panic
# ZERO-DAY OS Panic Button
# Triggered by: Fn+P, 'panic' command, or tamper-watch (violent movement)

PANIC_LOG="/opt/cardputer/panic.log"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

echo "[$TIMESTAMP] PANIC TRIGGERED" >> "$PANIC_LOG"

# Phase 1: Kill (0.1 seconds)
# Kill every offensive process by name
OFFENSIVE_PROCS="aircrack airodump airbase hostapd dnsmasq bettercap nmap"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS nikto whatweb sqlmap hcxdumptool hcxping wifite"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS bt-scan l2ping rfcomm obexftp"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS netcat ncat socat chisel rtl tcpdump"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS python perl ruby ir-replay ir-brute"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS john hydra gobuster responder arpspoof dnsspoof"
OFFENSIVE_PROCS="$OFFENSIVE_PROCS ffplay wifi-survey-log doh-proxy mac-rotate quick-c2 tunnel-mgr"

for proc in $OFFENSIVE_PROCS; do
    pkill -9 "$proc" 2>/dev/null
done

# Phase 2: Wipe (0.1 seconds)
history -c
history -w
rm -f ~/.bash_history
rm -f /root/.bash_history
rm -f /tmp/*
rm -rf /opt/cardputer/loot/* 2>/dev/null
tmux kill-server 2>/dev/null

# Phase 3: Sanitize (0.1 seconds)
# Clear screen, kill all tmux sessions, reset terminal
clear
reset
printf '\033[2J\033[H'

# Phase 4: Radio Silence
rfkill block all 2>/dev/null

# Phase 5: Display clean login
echo "ZERODAY OS v0.1 — Login:"
echo "================================"
echo "Password: "

echo "[$TIMESTAMP] PANIC COMPLETE" >> "$PANIC_LOG"
```

---

## 7. Power Management System

```bash
#!/bin/bash
# /usr/local/bin/power-mode
# Usage: power-mode [performance|balanced|stealth]

MODE="$1"
GOV="/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
FREQ="/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"

case "$MODE" in
    performance)
        # Full power — all cores, all radios
        echo "performance" > "$GOV"
        echo "1000000" > "$FREQ"                          # 1GHz
        for cpu in /sys/devices/system/cpu/cpu[1-3]/online; do echo 1 > "$cpu"; done
        rfkill unblock all
        echo 255 > /sys/class/backlight/*/brightness      # Full brightness
        cardputer-battery --profile=performance            # aggressive polling
        echo "PERFORMANCE: 1GHz quad, all radios, ~4hr battery"
        ;;
    balanced)
        # Middle ground — 800MHz, BT off, WiFi on
        echo "ondemand" > "$GOV"
        echo "800000" > "$FREQ"                            # 800MHz max
        for cpu in /sys/devices/system/cpu/cpu[2-3]/online; do echo 0 > "$cpu"; done  # 2 cores
        rfkill unblock wifi
        rfkill block bluetooth
        echo 180 > /sys/class/backlight/*/brightness
        echo "BALANCED: 800MHz dual, WiFi only, ~6hr battery"
        ;;
    stealth)
        # Maximum endurance — single core, no radios, dim screen
        echo "powersave" > "$GOV"
        echo "600000" > "$FREQ"                            # 600MHz max
        for cpu in /sys/devices/system/cpu/cpu[1-3]/online; do echo 0 > "$cpu"; done  # 1 core
        rfkill block all                                    # No RF emissions
        echo 80 > /sys/class/backlight/*/brightness        # Dim
        # Kill Xorg, switch to fbterm (saves ~25MB RAM)
        pkill Xorg
        fbterm &
        echo "STEALTH: 600MHz single, no radios, dim screen, ~10hr battery"
        ;;
    *)
        echo "Usage: power-mode [performance|balanced|stealth]"
        ;;
esac
```

---

## 8. Tamper Detection System

```bash
#!/bin/bash
# /usr/local/bin/tamper-watch
# Daemon: reads BMI270 IMU, triggers actions on movement
# Installed as systemd service: tamper-watch.service

IIO_DEV="/sys/bus/iio/devices/iio:device0"
CONFIG="/opt/cardputer/config/tamper.conf"

# Default thresholds
ACCEL_THRESHOLD=2.0     # g-force threshold for "violent" movement
MOVE_THRESHOLD=0.1      # g-force threshold for "any" movement
LOCK_ON_MOVE=true
WIPE_ON_VIOLENT=true

# Source config if exists
[ -f "$CONFIG" ] && source "$CONFIG"

while true; do
    # Read accelerometer values from BMI270
    AX=$(cat "$IIO_DEV/in_accel_x_raw" 2>/dev/null)
    AY=$(cat "$IIO_DEV/in_accel_y_raw" 2>/dev/null)
    AZ=$(cat "$IIO_DEV/in_accel_z_raw" 2>/dev/null)
    
    # Scale factor (BMI270 range ±16g, 16-bit)
    SCALE=$(cat "$IIO_DEV/in_accel_scale" 2>/dev/null || echo "0.009576")
    AX_G=$(echo "$AX * $SCALE" | bc -l)
    AY_G=$(echo "$AY * $SCALE" | bc -l)
    AZ_G=$(echo "$AZ * $SCALE" | bc -l)
    
    # Calculate total acceleration magnitude
    MAG=$(echo "sqrt($AX_G^2 + $AY_G^2 + $AZ_G^2)" | bc -l)
    
    # Detect movement
    DELTA=$(echo "$MAG - 9.81" | bc -l)
    ABS_DELTA=$(echo "if ($DELTA < 0) -1*$DELTA else $DELTA" | bc -l)
    
    if (( $(echo "$ABS_DELTA > $ACCEL_THRESHOLD" | bc -l) )); then
        # VIOLENT movement — wipe loot and history
        if [ "$WIPE_ON_VIOLENT" = true ]; then
            rm -rf /opt/cardputer/loot/* 2>/dev/null
            history -c && history -w
            rm -f ~/.bash_history
        fi
        echo "TAMPER: Violent movement detected" >> /opt/cardputer/panic.log
    elif (( $(echo "$ABS_DELTA > $MOVE_THRESHOLD" | bc -l) )); then
        # Gentle movement — lock terminal
        if [ "$LOCK_ON_MOVE" = true ]; then
            vlock  # Lock terminal, require password to unlock
        fi
        echo "TAMPER: Movement detected" >> /opt/cardputer/panic.log
    fi
    
    sleep 0.5
done
```

---

## 8a. Sub-GHz Radio System (CC1101)

The CC1101 Sub-GHz transceiver connects via SPI on the 2.54mm 14-pin expansion port. It allows receiving and transmitting signals in common ISM bands (315/433/868/915MHz), similar to the Flipper Zero's Sub-GHz feature.

### Hardware Wiring (2.54mm 14-Pin ExtPort)

```
  CC1101 Module    Cardputer Zero ExtPort
  ─────────────    ──────────────────────
  VCC (3.3V)   →   Pin 1 (3.3V)
  GND          →   Pin 2 (GND)
  MOSI         →   Pin 4 (SPI0 MOSI)
  MISO         →   Pin 5 (SPI0 MISO)
  SCK          →   Pin 6 (SPI0 SCLK)
  CSN          →   Pin 7 (SPI0 CE0)
  GDO0         →   Pin 9 (GPIO)
  GDO2         →   Pin 10 (GPIO)
```

### Script Architecture

```bash
# subghz-scan <freq_range>
# Scanning backends (in order of availability):
#   1. rtl_433 (RTL-SDR on USB-A) — wideband SDR reception
#   2. CC1101 on SPI (Python cc1101 library) — targeted frequency scan
#   3. rfcat (YardStick One on USB-A) — CC1111-based reception
# Frequency ranges: 300-348MHz, 387-464MHz, 779-928MHz
# Decodes: OOK, ASK, FSK, GFSK, MSK modulations
# Output: timestamped JSON + raw captures in /opt/cardputer/loot/rf/

# subghz-record <freq> [duration] [modulation]
# Records raw signal data for later replay
# Saves .raw files (timing data) and .json metadata
# Supported hardware: CC1101 SPI, YardStick One (rfcat), RTL-SDR

# subghz-replay <signal_file> [freq] [repeats]
# Replays a captured Sub-GHz signal
# Confirms before transmitting (legal compliance)
# Supported hardware: CC1101 SPI, YardStick One (rfcat)
# Default: 3 repeats at original frequency
```

### Pi-gen Stage Addition

```bash
# stage4/10-subghz-tools
pip3 install cc1101 rfcat 2>/dev/null || true
apt install -y --no-install-recommends rtl-433
# Install subghz-scan, subghz-record, subghz-replay to /usr/local/bin/
```

---

## 8b. NFC / RFID System (PN532)

The PN532 NFC/RFID module connects via I2C (or UART) on the Grove HY2.0-4P port. It supports reading, cloning, and emulating common NFC tag types.

### Hardware Wiring (Grove HY2.0-4P — I2C Mode)

```
  PN532 Module    Cardputer Zero Grove
  ────────────    ────────────────────
  VCC          →   Pin 1 (VCC 3.3V/5V)
  SDA          →   Pin 2 (SDA — I2C data)
  SCL          →   Pin 3 (SCL — I2C clock)
  GND          →   Pin 4 (GND)
  
  Set PN532 switches: SW1=ON, SW2=ON (I2C mode)
```

### Supported Tag Types

| Type | Read | Clone | Emulate | Notes |
|---|---|---|---|---|
| MIFARE Classic 1K/4K | ✓ | ✓ | ✓ | mfoc for key recovery |
| MIFARE Ultralight | ✓ | ✓ | — | Read NDEF, clone UID |
| NTAG213/215/216 | ✓ | ✓ | — | Amiibo, NFC tags |
| EM4100 (125kHz) | — | — | ✓ | Requires separate 125kHz module |

### Script Architecture

```bash
# nfc-read [output]
# Auto-detects PN532 on I2C, UART, or USB (ACR122U)
# Reads: UID, tag type, NDEF records, MIFARE sectors
# Saves to /opt/cardputer/loot/rf/nfc_read_*.json
# Fallback: mfoc for MIFARE Classic key recovery

# nfc-clone <uid_or_dump> [output]
# Clone by UID: writes UID to blank writable tag
# Clone from dump: writes .mfd dump to blank tag
# Supports: nfc-mfultralight, pm3, nfcpy
# WARNING: Only clone tags you own/authorize

# nfc-emulate <type_or_uid> [duration]
# Types: mifare, ntag, em4100, or custom UID
# Makes Cardputer Zero act as an NFC tag
# Requires: Proxmark3 (USB) or PN532 (emulation mode)
# Duration: 0 = until Ctrl+C
```

### Pi-gen Stage Addition

```bash
# stage4/11-nfc-tools
apt install -y --no-install-recommends libnfc-bin mfoc pcscd
pip3 install nfcpy 2>/dev/null || true
# Install nfc-read, nfc-clone, nfc-emulate to /usr/local/bin/
```

---

## 8c. Meshtastic Mesh Networking

Meshtastic provides encrypted LoRa mesh networking for off-grid communication. A Meshtastic-compatible module connects via UART on the Grove port.

### Hardware Wiring (Grove HY2.0-4P — UART Mode)

```
  Meshtastic      Cardputer Zero Grove
  Module          (switch to UART mode)
  ──────────      ────────────────────
  VCC          →   Pin 1 (VCC 3.3V/5V)
  TX           →   Pin 2 (RX — UART receive)
  RX           →   Pin 3 (TX — UART transmit)
  GND          →   Pin 4 (GND)
  
  Note: Grove port must be switched from I2C to UART mode
  (conflicts with PN532 NFC — not simultaneous)
```

### mesh-chat Architecture

```bash
# mesh-chat install [--port /dev/ttyUSB0] [--baud 115200]
# Installs meshtastic Python CLI, configures serial port
# Sets node name, region, and channel config

# mesh-chat send <node|channel> <message>
# Send text to a specific node or broadcast to channel
# Supports: All (broadcast), channel number, node ID

# mesh-chat listen [duration]
# Monitor all incoming messages (one-time or continuous)
# Saves received messages to /opt/cardputer/loot/recon/mesh_*.log

# mesh-chat chat [channel]
# Interactive IRC-like chat on specified channel
# Shows real-time message feed with node names

# mesh-chat nodes
# List discovered nodes with: name, SNR, hop count, GPS, battery

# mesh-chat exfil <file>
# Encrypt and transmit a file over mesh in chunks
# Receiver reassembles: mesh-chat receive [output]

# mesh-chat info
# Show local node info: name, battery, GPS, region, firmware
```

**Important:** Meshtastic is an off-grid **communication tool**, not a C2 framework. It provides:
- Encrypted peer-to-peer messaging
- No internet required (LoRa radio)
- Data exfiltration through mesh (when all local networks are monitored)
- Team coordination during pentests

---

## 8d. Captive Portal (Evil Twin)

The `wifi-evil-twin` script creates a rogue access point with a captive portal that hijacks DNS and harvests credentials. It's the most social-engineering-capable tool in the arsenal.

### Architecture

```
  victim device                       Cardputer Zero
  ┌──────────┐                    ┌──────────────────────┐
  │ connects  │                    │  hostapd (AP)        │
  │ to ESSID  │ ←── WiFi ──────── │  wlan0: 10.0.0.1    │
  │           │                    │                      │
  │ browser   │ ←── DNS ────────  │  dnsmasq (DHCP+DNS) │
  │ redirects │     hijack        │  all queries → .1    │
  │           │                    │                      │
  │ sees      │ ←── HTTP ─────── │  Python HTTP server  │
  │ login     │     portal        │  (captive portal)    │
  │ page      │                    │  POST → log creds   │
  │           │                    │                      │
  │ submits   │ ─── POST ──────→  │  credentials saved   │
  │ username  │                    │  to /opt/loot/       │
  │ + password│                    │                      │
  │           │                    │  iptables NAT        │
  │ internet  │ ←── NAT ────────  │  eth0 → wlan0        │
  │ works     │     (after login) │  (victim gets net)   │
  └──────────┘                    └──────────────────────┘
```

### Portal Types

| Type | Description | Use Case |
|---|---|---|
| `wifi` | Fake WiFi login page | Hotels, cafes, airports |
| `corporate` | Fake VPN login page | Corporate environments |
| `social` | Fake social media login | Public spaces |
| `custom` | Serve your own HTML from `/opt/cardputer/config/captive/` | Advanced |

### Credential Flow

1. Victim connects to rogue AP (same ESSID as target)
2. DNS hijack redirects all HTTP/HTTPS to portal
3. Victim sees convincing login page
4. Credentials POSTed and logged: `[timestamp] IP - Username: X Password: Y`
5. Victim redirected to "Connected!" page
6. NAT enables real internet access (victim doesn't suspect)
7. All credentials saved to `/opt/cardputer/loot/recon/captive_creds_*.log`

---

## 8e. Boot Animation

The boot animation (`zeroday-bootanim`) plays on the ST7789 LCD during system startup, before Xorg/i3. It provides a cyberpunk aesthetic and system status feedback.

### Animation Phases

```
  [0.0s] Glitch noise     Random glyphs flicker (green/red/cyan)
  [0.5s] Banner reveal    ZERO-DAY banner appears with corruption artifacts
  [1.5s] Boot sequence     Subsystem init lines type out (kernel, memory, display...)
  [2.5s] Warning          Authorized-use reminder
  [3.0s] Ready             Keybinding hints + blinking cursor
  [3.5s] Final glitch     Brief screen tear, then Xorg launches
```

### Technical Implementation

```bash
# /usr/local/bin/zeroday-bootanim
# Called by /usr/local/bin/zeroday-boot before startx
# Uses pure bash + ANSI escape codes (no Python dependency)
# Phases: glitch noise → banner reveal → boot log → warning → ready

# Disable options:
#   touch /etc/zeroday/no-bootanim    # Permanent skip
#   ZERODAY_NO_BOOTANIM=1             # Skip for this boot
```

The animation is integrated into `zeroday-boot.service` which runs at multi-user.target. The boot sequence is:

```
systemd → zeroday-boot.service → zeroday-bootanim → Xorg + i3 → cyber_launcher TUI
```

---

## 8f. M5MonsterC5 Integration

The M5MonsterC5 is an ESP32C5-based add-on board running JanOS/projectZero firmware. It connects to the Cardputer Zero via USB-A or UART serial and provides a dedicated WiFi attack radio with its own suite of offensive tools.

### Communication

- **Interface:** UART/USB serial at 115200 baud, 8N1
- **Protocol:** Line-based text commands terminated with `\r\n`
- **Board detection:** `ping` command → expects `pong` response
- **Auto-detection:** `monsterctl` scans `/dev/ttyUSB*`, `/dev/ttyACM*`, `/dev/ttyAMA0`

### Script Architecture

```bash
# /usr/local/bin/monsterctl
# Wraps all serial communication with the M5MonsterC5 board
# Auto-detects the serial port on first run
# Sends commands as line-based text over UART/USB serial
# Parses responses and displays to user

# Usage:
#   monsterctl <subcommand> [args]

# Subcommands:
#   ping              - Verify board connection (expects pong)
#   scan              - Scan WiFi networks
#   select <id> [id]  - Select target AP(s) by scan index
#   deauth            - Deauth attack on selected AP
#   evil_twin         - Evil twin AP on selected target
#   sae_overflow      - WPA3 SAE overflow attack
#   handshake         - Capture WPA/WPA2 handshake
#   sniffer           - WiFi packet sniffer
#   blackout          - Mass deauth all visible APs
#   sniffer_dog       - Follow and sniff a specific client
#   karma             - Karma probe response attack
#   beacon_spam       - Flood beacons with random SSIDs
#   wardrive          - Wardrive scan with GPS
#   nmap <target>     - Port scan via board
#   arp_poison <target> - ARP poisoning
#   rogue_ap          - Standalone rogue AP
#   deauth_detect     - Detect deauth attacks
#   passwords         - Show captured credentials
#   hosts             - Show discovered hosts
#   probes            - Show captured probe requests
#   wifi_connect      - Connect board to WiFi
#   wifi_disconnect   - Disconnect board from WiFi
#   gps               - Show GPS coordinates
#   channel_time <ms> - Set channel dwell time
#   list_sd           - List files on board SD card
#   list_html         - List captive portal HTML pages
#   stop              - Stop all running attacks
#   status            - Show board status
#   flash <web|local|cardputer> - Flash firmware
```

### Serial Protocol Details

```
Cardputer Zero                          M5MonsterC5
──────────────                          ────────────
─────────────────────────────────────►  UART/USB (115200 8N1)
  ping\r\n                           →  pong\r\n
  scan\r\n                            →  scan results (line-delimited)
  select 1 3\r\n                      →  OK\r\n
  deauth\r\n                          →  [attack output]\r\n
  stop\r\n                            →  OK\r\n
  status\r\n                          →  [status JSON]\r\n
```

### Board Wiring Options

| Connection | Pins / Port | Notes |
|---|---|---|
| **USB-A** | USB-A port on Cardputer Zero | Auto-detected as `/dev/ttyACM*` or `/dev/ttyUSB*` |
| **UART** | GPIO TX/RX (Grove or ExtPort) | `/dev/ttyAMA0`, requires `serial0` config |

The `monsterctl` script auto-detects the port on startup. No manual configuration needed.

### Interfaces

The M5MonsterC5 board has two interfaces on ZERO-DAY OS:

| Interface | Type | Use Case |
|---|---|---|
| `monsterctl` | CLI / automation | One command per invocation, scriptable, pipeline-friendly |
| `install-janos` | Interactive TUI | Menu-driven, visual, real-time monitoring and browsing |

`monsterctl` is the CLI/automation interface. `install-janos` launches the JanOS-app TUI (see section 8g), which provides a full interactive menu for scanning, attacking, wardriving, and browsing captured data.

### Links

- [M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV) — Hardware and firmware
- [projectZero](https://github.com/C5Lab/projectZero) — JanOS/projectZero firmware

---

## 8g. JanOS-app Integration

The JanOS-app is a Python TUI application that provides an interactive front-end for the M5MonsterC5 board. It is **not included in the base image** (too large for the 512MB RAM constraint) and is installed on-demand via `install-janos`.

### Script Architecture

```bash
# /usr/local/bin/install-janos
# Manages the JanOS-app lifecycle: install, run, update, status
#
# Subcommands:
#   install              - Clone JanOS-app from GitHub, install pyserial
#   run [/dev/ttyUSB0]  - Launch the interactive TUI (auto-detect serial port)
#   update               - Pull latest from GitHub
#   status               - Check if JanOS-app is installed
#
# Installation directory: /opt/cardputer/janos-app/
# Dependencies: pyserial (lightweight, <5MB RAM)
# Alias: monsterctl janos → install-janos run
```

### Communication

The JanOS-app communicates with the M5MonsterC5 board over UART at 115200 baud using the same command set as `monsterctl`. The TUI translates menu selections into serial commands:

```
JanOS-app TUI                     M5MonsterC5
────────────────                  ───────────
Menu: "Scan"            →        scan\r\n
Menu: "Select target"   →        select <id>\r\n
Menu: "Deauth"          →        deauth\r\n
Menu: "Evil Twin"       →        evil_twin\r\n
Menu: "Wardrive"        →        wardrive\r\n
Menu: "Sniffer"         →        sniffer\r\n
...
Response parsed and rendered in TUI
```

### Why not bundled in the image

| Factor | Reason |
|---|---|
| **RAM** | Python TUI adds ~25MB; every MB counts on 512MB |
| **Image size** | JanOS-app repo adds ~15MB to the 3.5GB image |
| **Update frequency** | JanOS-app updates independently of ZERO-DAY OS |
| **Not always needed** | CLI users may never launch the TUI |

The `install-janos` script downloads the TUI on first use, keeping the base image lean.

### Relationship to monsterctl

- **`monsterctl`** — CLI/automation interface. One command per invocation. Use in scripts, pipes, keybindings.
- **`install-janos run`** — Interactive TUI. Menu-driven, visual, real-time. Use for exploration and monitoring.
- **`monsterctl janos`** — Alias for `install-janos run`.

Both send the same serial commands to the M5MonsterC5 board. Choose based on workflow.

### Repository

- [JanOS-app](https://github.com/D3h420/JanOS-app) — Interactive TUI for M5MonsterC5

---

## 8h. Ragnar Reconnaissance Scripts

[Ragnar](https://github.com/PierreGode/Ragnar) is a full Python recon platform requiring 2–8GB RAM. ZERO-DAY OS provides three lightweight scripts inspired by Ragnar's capabilities, each running in <50MB RAM using pure bash + curl + jq.

### ragnar-scan

```bash
# /usr/local/bin/ragnar-scan
# Autonomous 3-phase network reconnaissance
# Phase 1: Discover — ARP scan + ping sweep (find live hosts)
# Phase 2: Scan — Nmap port scan with selected profile
# Phase 3: Summarize — Collate results into human-readable report
#
# Usage: ragnar-scan [interface] [quick|full|vuln|stealth]
# Output: /opt/cardputer/loot/recon/ragnar_scan_<timestamp>/
#   ├── hosts.txt          # Discovered hosts
#   ├── scan.xml           # Nmap XML output
#   ├── scan.nmap          # Nmap text output
#   └── summary.txt        # Human-readable summary
#
# Profiles:
#   quick   - Top 1000 ports, ~30 seconds per host
#   full    - All 65535 ports, ~5 minutes per host
#   vuln    - Nmap vuln scripts, ~10 minutes per host
#   stealth - SYN scan, no ping, fragmented packets
#
# RAM usage: ~30-40MB (bash + nmap + jq)
```

### threat-intel

```bash
# /usr/local/bin/threat-intel
# CVE and CISA Known Exploited Vulnerabilities lookup
# Pure bash + curl + jq, no Python dependencies
#
# Subcommands:
#   cve <CVE-ID>          - Look up CVE details from NVD
#   search <keyword>      - Search CISA KEV by keyword
#   check <service> [ver] - Check known vulns for a service/version
#   recent                - Show recent CISA KEV additions (last 30 days)
#
# API endpoints:
#   NVD: https://services.nvd.nist.gov/rest/json/cves/2.0
#   CISA KEV: https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json
#
# RAM usage: ~10-20MB (curl + jq)
```

### device-classify

```bash
# /usr/local/bin/device-classify
# Classify network devices from nmap XML output
# Uses vendor OUI + service fingerprinting
#
# Usage: device-classify [nmap_xml_file]
#   If no file given, uses latest scan from /opt/cardputer/loot/recon/
#
# Categories:
#   Network Infra - Routers, switches, APs (Cisco, Juniper, Aruba)
#   IoT Devices   - Cameras, smart TVs, printers (Hikvision, Samsung, HP)
#   Workstations  - Windows, Linux, macOS (by SMB/SSH fingerprint)
#   Servers       - Web, mail, database (by open ports + banners)
#
# RAM usage: ~5-10MB (bash + jq)
```

### Why not full Ragnar

| Resource | Cardputer Zero | Ragnar requirement |
|---|---|---|
| RAM | 512MB (382MB free) | 2–8GB |
| Python stack | Minimal | ML libs, Scikit-learn, etc. |
| Web dashboard | No browser | Flask/Dash UI |
| Nuclei + ZAP | No Go runtime | Required for full scans |

For the complete Ragnar experience, run it on a separate machine (8GB+ RAM) and use ZERO-DAY OS as the hands-on attack tool. Feed ragnar-scan output to Ragnar for AI-powered analysis.

---

## 9. Flipper TUI — Technical Design

```
Language:    Python 3
Framework:   Pygame (SDL2 backend, no X11 dependency at runtime)
Renderer:    SDL2 → DRM/KMS or X11 backend
Screen Size: 320x170 (1.9" ST7789v3)

File: /usr/local/bin/cyber_launcher
```

### Class Structure

```python
# cyber_launcher.py — Architecture Overview

import pygame
import subprocess
import threading
import socket
import signal
import os
import re
import shlex
import select

class CyberLauncher:
    """Main Flipper-style Pygame GUI launcher"""
    
    # Screen: 320x170, 30 FPS target
    # States: SPLASH → HOME → LIST → ACTION | PROMPT | WALKIE_TALKIE | MEDIA_PLAYER
    
    CATEGORIES = [  # 12 categories, 4x3 grid
        "WIFI", "M5MONSTER", "NET", "BT",
        "IR", "CAM", "PAYLD", "RADIO",
        "MEDIA", "SHELL", "SYS", "OPEN"
    ]
    
    # 12 categories fill a 4×3 grid perfectly
    # Navigation: Arrow keys, Enter, Esc, Tab (in PROMPT mode)
    # Walkie-Talkie: Space = PTT, UDP broadcast on port 42420
    # Media Player: Left/Right = change station, Esc = stop
```

### Rendering System

```python
# Pygame rendering — direct framebuffer drawing, no CSS

# Color Palette (Kali Nethunter-inspired)
BG_COLOR       = (5, 8, 15)        # Deep abyss blue-black
DIM_BG         = (15, 20, 35)       # Dark panel background
TEXT_PRIMARY    = (43, 204, 255)     # Kali Cyan
TEXT_WHITE      = (240, 250, 255)    # Near-white
CMD_FLAG        = (255, 75, 75)      # Kali Red

# Category colors — each of 12 categories has a unique color
# WIFI=Cyan, M5MONSTER=Red, NET=Blue, BT=SoftBlue,
# IR=Orange, CAM=Pink, PAYLD=Gold, RADIO=Purple,
# MEDIA=Green, SHELL=Red, SYS=Grey, OPEN=Cyan

# Fonts: Terminus (monospace, bitmap) preferred, fallback to system monospace
# All drawing via pygame.draw.rect(), pygame.draw.line(), pygame.font.SysFont()

# Screens: 320x170 at 30 FPS
# HOME:     4×3 grid of categories (80px cells)
# LIST:     Scrollable tool list with colored sidebar
# ACTION:   Confirmation dialog for command execution
# PROMPT:   Multi-field argument input with validation
# WALKIE:   PTT radio (UDP broadcast, port 42420)
# MEDIA:    Radio station selector + local music player
```

---

## 10. One-Key Script Architecture

Every hacking command is a standalone bash script in `/usr/local/bin/`. They follow a consistent pattern:

```bash
#!/bin/bash
# /usr/local/bin/<tool-name>
# Every script follows this template:

# 1. Header with usage
# 2. Argument validation (or interactive prompt if args missing)
# 3. Check for root (if needed)
# 4. Check for required tools/binaries
# 5. Create output directory if needed
# 6. Execute the command
# 7. Report results and save to /opt/cardputer/loot/
```

### Example: wifi-handshake

```bash
#!/bin/bash
# /usr/local/bin/wifi-handshake
# Capture WPA handshakes from target AP
# Usage: wifi-handshake <iface> [bssid] [channel]

IFACE="${1:-wlan0}"
BSSID="$2"
CHAN="$3"
OUTDIR="/opt/cardputer/handshakes"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$OUTDIR"

# If no BSSID provided, scan for targets
if [ -z "$BSSID" ]; then
    echo "[*] Scanning for WiFi networks on $IFACE..."
    sudo ip link set "$IFACE" down
    sudo iw dev "$IFACE" set type monitor
    sudo ip link set "$IFACE" up
    sudo airodump-ng "$IFACE" --output-format csv -w /tmp/zeroday_scan
    echo ""
    echo "[!] Select a target BSSID and channel, then re-run:"
    echo "    wifi-handshake $IFACE <BSSID> <CHANNEL>"
    echo ""
    echo "[*] Results saved to /tmp/zeroday_scan-01.csv"
    exit 0
fi

# Validate interface is in monitor mode
IFTYPE=$(iw dev "$IFACE" type 2>/dev/null | awk '{print $2}')
if [ "$IFTYPE" != "monitor" ]; then
    echo "[*] Setting $IFACE to monitor mode..."
    sudo ip link set "$IFACE" down
    sudo iw dev "$IFACE" set type monitor
    sudo ip link set "$IFACE" up
fi

# Set channel
sudo iw dev "$IFACE" set channel "$CHAN"

# Capture handshake
OUTFILE="$OUTDIR/${BSSID//:/}_ch${CHAN}_${TIMESTAMP}"
echo "[*] Capturing handshake for $BSSID on channel $CHAN..."
echo "[*] Saving to $OUTFILE.cap"
echo "[*] Press Ctrl+C when handshake captured"
echo ""

sudo airodump-ng "$IFACE" --bssid "$BSSID" --channel "$CHAN" \
    --output-format pcap -w "$OUTFILE"

echo ""
echo "[*] Handshake capture stopped."
echo "[*] Check: $OUTFILE.cap"
echo "[*] Crack with: wifi-crack $OUTFILE.cap"
```

---

## 11. USB Gadget Mode — The Silent Vector

The Cardputer Zero's USB-C can switch from **Host** to **Device** mode via the physical `USB SW` switch:

```bash
#!/bin/bash
# /usr/local/bin/usb-gadget-mode
# Switch USB-C between Host and Device modes
# Requires USB_SW to be set to "Device" position on the board

MODE="$1"

case "$MODE" in
    hid)
        # USB Keyboard (Rubber Ducky mode)
        # Enumerate as HID keyboard to victim PC
        # Payload from /opt/cardputer/payloads/ducky.txt
        modprobe libcomposite
        mkdir -p /sys/kernel/config/usb_gadget/zeroday
        cd /sys/kernel/config/usb_gadget/zeroday
        echo "0x1d6b" > idVendor    # Linux Foundation
        echo "0x0104" > idProduct    # Multifunction Composite Gadget
        echo "0x0100" > bcdDevice
        echo "0x0200" > bcdUSB
        mkdir -p strings/0x409/serialnumber
        mkdir -p strings/0x409/manufacturer
        mkdir -p strings/0x409/product
        echo "0123456789" > strings/0x409/serialnumber
        echo "ZeroDay" > strings/0x409/manufacturer
        echo "Cardputer Zero" > strings/0x409/product
        mkdir -p functions/hid.usb0
        echo 1 > functions/hid.usb0/protocol
        echo 1 > functions/hid.usb0/subclass
        echo 8 > functions/hid.usb0/report_length
        # Report descriptor for keyboard
        echo -ne \\x05\\x01\\x09\\x06\\xa1\\x01\\x05\\x07\\x19\\xe0\\x29\\xe7\\x15\\x00\\x25\\x01\\x75\\x01\\x95\\x08\\x81\\x02\\x95\\x01\\x75\\x08\\x81\\x01\\x95\\x05\\x75\\x01\\x05\\x08\\x19\\x01\\x29\\x05\\x91\\x02\\x95\\x01\\x75\\x03\\x91\\x01\\x95\\x06\\x75\\x08\\x15\\x00\\x25\\x65\\x05\\x07\\x19\\x00\\x29\\x65\\x81\\x00\\xc0 > functions/hid.usb0/report_desc
        mkdir -p configs/c.1/strings/0x409
        echo "HID" > configs/c.1/strings/0x409/configuration
        ln -s functions/hid.usb0 configs/c.1/
        ls /sys/class/udc > UDC
        echo "[*] USB HID keyboard mode active"
        echo "[*] Execute payload: ducky-exec /opt/cardputer/payloads/ducky.txt"
        ;;
    
    serial)
        # USB Serial (debug console)
        modprobe g_serial
        echo "[*] USB serial mode active on /dev/ttyGS0"
        echo "[*] Connect from host: screen /dev/ttyACM0 115200"
        ;;
    
    ncm)
        # USB Network (for host networking)
        modprobe g_ncm
        echo "[*] USB network mode active"
        echo "[*] Configure: ifconfig usb0 10.0.0.1 netmask 255.255.255.0"
        ;;
    
    mass)
        # USB Mass Storage (exfil mode)
        # Expose /opt/cardputer/loot/ as USB drive
        BACKING_FILE="/opt/cardputer/usbgadget.img"
        dd if=/dev/zero of="$BACKING_FILE" bs=1M count=256
        mkfs.vfat "$BACKING_FILE"
        modprobe g_mass_storage file="$BACKING_FILE" stall=0
        echo "[*] USB mass storage mode active"
        ;;
    
    off)
        # Disable gadget mode, return to host
        echo "" > /sys/kernel/config/usb_gadget/zeroday/UDC 2>/dev/null
        rmdir /sys/kernel/config/usb_gadget/zeroday/functions/hid.usb0 2>/dev/null
        rmdir /sys/kernel/config/usb_gadget/zeroday 2>/dev/null
        rmmod g_serial g_ncm g_mass_storage libcomposite 2>/dev/null
        echo "[*] Gadget mode disabled, returned to host"
        ;;
    
    *)
        echo "Usage: usb-gadget-mode [hid|serial|ncm|mass|off]"
        echo ""
        echo "  hid     - USB keyboard (Rubber Ducky mode)"
        echo "  serial  - USB serial console (debug)"
        echo "  ncm     - USB network (host networking)"
        echo "  mass    - USB mass storage (exfil)"
        echo "  off     - Disable gadget, return to host"
        echo ""
        echo "NOTE: Physical USB_SW switch must be in 'Device' position"
        ;;
esac
```

---

## 12. OpenCode Integration — Implementation

OpenCode is installed as a standalone binary and wrapped by a tmux-based IDE script:

```bash
#!/bin/bash
# /usr/local/bin/opencode-session
# tmux split-screen IDE: OpenCode + live console

SESSION="opencode"
WORKSPACE="/opt/cardputer/workspace"

mkdir -p "$WORKSPACE"

# If session exists, attach; otherwise create
if tmux has-session -t "$SESSION" 2>/dev/null; then
    tmux attach -t "$SESSION"
    exit 0
fi

# Determine working directory
DIR="${1:-$WORKSPACE}"
FILE="$2"

cd "$DIR" || exit 1

# Create new tmux session
tmux new-session -d -s "$SESSION" -c "$DIR"

# Split horizontally: 70% top, 30% bottom
tmux split-window -v -p 30 -t "$SESSION" -c "$DIR"

# Top pane: OpenCode (or the file specified)
if [ -n "$FILE" ] && [ -f "$DIR/$FILE" ]; then
    tmux send-keys -t "$SESSION:0.0" "opencode $DIR/$FILE" Enter
else
    tmux send-keys -t "$SESSION:0.0" "opencode" Enter
fi

# Bottom pane: bash
tmux send-keys -t "$SESSION:0.1" "" Enter

# Select top pane
tmux select-pane -t "$SESSION:0.0"

# Attach
tmux attach -t "$SESSION"
```

### OpenCode Configuration

```
# /opt/cardputer/config/opencode/settings.json
{
    "workspace": "/opt/cardputer/workspace",
    "terminal": {
        "shell": "/bin/bash",
        "fontSize": 8
    },
    "editor": {
        "fontSize": 8,
        "tabSize": 2,
        "wordWrap": true,
        "theme": "dark"
    },
    "keybindings": {
        "save": "Ctrl+S",
        "quit": "Ctrl+Q",
        "terminal": "Ctrl+`",
        "newFile": "Ctrl+N"
    }
}
```

---

## 13. Custom Kernel Configuration

```
# Key kernel configs for ZERO-DAY OS (bcm2837_defconfig with modifications)

# ─── Must Enable ───
CONFIG_SPI=y                          # ST7789v3 LCD
CONFIG_SPI_BCM2835=y                  # SPI controller
CONFIG_I2C=y                          # IMU, battery, RTC, keyboard
CONFIG_I2C_BCM2835=y                  # I2C controller
CONFIG_I2C_HID=y                      # HID over I2C (keyboard)
CONFIG_SND=y                          # Audio subsystem
CONFIG_SND_BCM2835=y                   # BCM audio
CONFIG_SND_SOC_ES8389=y               # ES8389 codec
CONFIG_VIDEO_IMX219=y                 # Camera
CONFIG_MEDIA_CONTROLLER=y             # libcamera
CONFIG_V4L2_MEM2MEM=y                  # Hardware video encode/decode
CONFIG_HWMON=y                        # Hardware monitoring
CONFIG_BATTERY_BQ27220=y              # Battery fuel gauge
CONFIG_RTC_DRV_RX8130=y               # Real-time clock
CONFIG_BMI270=y                       # IMU
CONFIG_BMI270_I2C=y                   # IMU over I2C
CONFIG_LIRC=y                         # IR transceiver
CONFIG_RC_CORE=y                      # Remote controller core
CONFIG_USB_CONFIGFS=y                 # USB gadget mode
CONFIG_USB_CONFIGFS_HID=y             # HID gadget
CONFIG_USB_CONFIGFS_SERIAL=y           # Serial gadget
CONFIG_USB_CONFIGFS_NCM=y             # Network gadget
CONFIG_USB_CONFIGFS_MASS_STORAGE=y    # Mass storage gadget
CONFIG_BT=y                           # Bluetooth
CONFIG_BT_BCM=y                       # BCM Bluetooth
CONFIG_BT_HCIUART=y                   # BT over UART
CONFIG_CFG80211=y                     # Wireless config
CONFIG_BRCMFMAC=y                     # WiFi driver
CONFIG_NET_VENDOR_REALTEK=y           # RTL8152 (USB-Ethernet)

# ─── Must Disable (save RAM/kernel size) ───
CONFIG_DRM=n                          # No DRM (using fbdev)
CONFIG_SOUND_OSS_CORE=n               # No OSS audio
CONFIG_FB_RPISENSEDISPLAY=n            # No sense hat display
CONFIG_USB_PRINTER=n                  # No printer support
CONFIG_JOYSTICK=n                      # No joystick
CONFIG_HID_LOGITECH=n                 # No Logitech specific drivers
CONFIG_SND_USB_AUDIO=n                # No USB audio (use built-in)

# ─── Size Optimization ───
CONFIG_MODULES=y                       # Enable modules (load on demand)
CONFIG_MODULE_UNLOAD=y                # Allow module unloading
CONFIG_MODULE_FORCE_UNLOAD=y          # Force unload to free RAM
# Unused modules built as modules, not built-in
# Total kernel image target: <8MB compressed
```

---

## 14. First Boot Wizard

```
┌─────────────────────────────────────┐
│     ZERO-DAY OS  ·  FIRST BOOT      │
│                                     │
│  [1/5] Change root password         │
│  Current: zeroday                    │
│  New password: ****                  │
│                                     │
│  [2/5] Operator user password       │
│  New password: ****                  │
│                                     │
│  [3/5] WiFi Configuration           │
│  (Skip to stay offline)             │
│  SSID: ____________                  │
│  PSK:  ____________                  │
│                                     │
│  [4/5] Date & Time                  │
│  Timezone: UTC                      │
│  NTP: pool.ntp.org                  │
│                                     │
│  [5/5] System Info                  │
│  Battery: 87% (4h 12m remaining)    │
│  SD Card: 28.3GB free              │
│  Kernel: 6.1.x-zeroday              │
│                                     │
│  [Enter] Start ZERO-DAY OS          │
│                                     │
└─────────────────────────────────────┘
```

---

## 15. MOTD (Login Banner)

```
╔══════════════════════════════════════════════════╗
║                                                  ║
║   ZERO-DAY OS  v0.1-pre                         ║
║   M5Stack Cardputer Zero (CM0)                  ║
║                                                  ║
║   Battery: ████████░░ 82%   |  CPU: 800MHz     ║
║   WiFi: OFF  |  BT: OFF  |  Eth: DOWN          ║
║   RAM: 131M/512M  |  Disk: 28.1G free          ║
║                                                  ║
║   Fn+Tab  TUI    Fn+P  Panic    Fn+O  OpenCode ║
║   Fn+N    Nmap   Fn+B  BT Scan  Fn+S  Shell    ║
║   Fn+W    WiFi   Fn+C  Camera   Fn+I  IR       ║
║                                                  ║
╚══════════════════════════════════════════════════╝
```

---

## 16. Build Pipeline

```
Developer Machine (x86 Linux)
         │
         ▼
┌─────────────────────┐
│  docker build        │  ← pi-gen runs inside Docker
│  ./build-docker.sh   │     for ARM cross-compilation
│                      │
│  Stage 0: debootstrap│  5 min
│  Stage 1: base system│  10 min
│  Stage 2: networking │  8 min
│  Stage 3: ZERO-DAY   │  15 min (core customization)
│  Stage 4: hacking    │  40 min (apt installs from Kali)
│  Stage 5: zero-touch │  5 min (cleanup, first-boot)
│                      │
│  Total: ~80-120 min   │
└─────────────────────┘
         │
         ▼
  pi-gen/deploy/zeroday-os-.img
         │
         ▼
┌─────────────────────┐
│  Flash to microSD   │  ← dd or BalenaEtcher
│  (32GB minimum)      │
│                      │
│  sudo dd if=zeroday-os.img \
│    of=/dev/sdX bs=4M \
│    status=progress   │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  Boot on Cardputer  │  ← First-boot wizard runs
│  Zero               │     auto-login → i3 → TUI
│                      │
│  ~7 seconds to TUI   │
└─────────────────────┘
```

---

## 17. Pre-Release Blockers

The Cardputer Zero has not yet shipped. These items require final hardware or pinout documentation:

| Blocker | Status | Impact |
|---|---|---|
| **Keyboard matrix GPIO pinout** | Pending M5Stack release | Cannot build `cardputer-kbd-overlay.dts` — keyboard won't work |
| **ST7789v3 wiring (SPI bus, CS, DC, RST)** | Pending | Cannot build LCD overlay — no display |
| **ES8389 I2C address + I2S wiring** | Pending | No audio — mic, speaker, headphone jack dead |
| **IMX219 camera enable** | Pending | Camera inoperative |
| **BMI270 I2C bus/address** | Pending | No IMU / tamper detection |
| **BQ27220 I2C address** | Pending | No battery reporting |
| **RX8130CE I2C address** | Pending | No hardware RTC |
| **IR TX/RX GPIO pins** | Pending | No IR hacking |
| **USB-C device mode wiring** | Pending | No USB gadget mode (Rubber Ducky) |
| **Official CM0 device tree** | Pending | Base DT may need patches |

**What we CAN build now:**
- All one-key hacking scripts (pure bash, no hardware dependency)
- Sub-GHz scripts (subghz-scan, subghz-record, subghz-replay)
- NFC scripts (nfc-read, nfc-clone, nfc-emulate)
- Meshtastic mesh-chat wrapper
- Captive portal evil twin (wifi-evil-twin)
- Boot animation (zeroday-bootanim)
- The TUI app (`cyber_launcher`)
- i3 configuration and keybindings
- Panic system
- Power management scripts
- OpenCode session wrapper
- pi-gen stage 3 (base system customization)
- Kernel config (based on BCM2837, will need overlay adjustments)
- USB gadget mode scripts (framework only, needs hardware test)
- RTL8821CU dongle setup script
- First-boot wizard
- All documentation

**What we MUST WAIT for:**
- Device tree overlays (needs pinout)
- Keyboard driver (needs pinout)
- LCD driver (needs pinout)
- Audio driver (needs pinout)
- Final testing on real hardware

---

## 18. Recommended microSD Card

| Spec | Minimum | Recommended |
|---|---|---|
| Capacity | 16 GB | 32 GB+ |
| Speed Class | Class 10 (U1) | U3 / V30 |
| Form Factor | microSDHC | microSDHC |
| Endurance | Standard | High Endurance (for constant writes) |

The OS image targets ~3.5 GB. With wordlists (~200 MB), you have room for captured data on a 16 GB card. A 32 GB card gives comfortable headroom.

**Important:** The OS mounts `/tmp`, `/var/log`, and `/var/tmp` as tmpfs (RAM disks) to minimize SD card writes. Loot is written to `/opt/cardputer/loot/` which is on the SD card — consider `sync` mounts for critical data.

---

<p align="center">
<strong>This is the blueprint. Now we build.</strong>
</p>