# ZERO-DAY OS

<p align="center">
  <img src="assets/logo.png" alt="ZERO-DAY OS Logo" width="480">
</p>

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

ZERO-DAY OS v2.0 turns the M5Stack Cardputer Zero — a quad-core ARM64 box with WiFi, BT, IR, a camera, a battery, and a built-in keyboard — into a pocketable offensive security weapon. Every byte of this distro is optimized for the constraints of 512MB RAM and a 1.9" screen. No desktop. No bloat. No compromises.

[![Release v2.0](https://img.shields.io/github/v/release/jayis1/Zero-Day-OS-?label=latest%20release)](https://github.com/jayis1/Zero-Day-OS-/releases/latest)

---

## What Makes This Different

You can install Kali on a Raspberry Pi. That's not what this is.

| Stock Pi + Kali | ZERO-DAY OS |
|---|---|
| Boots into a desktop you can't use on 1.9" | Boots straight into a Textual TUI launcher |
| 2GB+ RAM just for the DE | ~60MB idle, 512MB total — 450MB for tools |
| Mouse required | 46-key Omni-Key system — zero mouse needed |
| Tools are menu items you click | Tools are **2 keystrokes away** from anywhere |
| CLI needed for file management | Native D-Pad File Explorer built into TUI |
| No hardware awareness | IR, camera, IMU, battery — all weaponized |
| Close lid, pray | Press `Fn + P` — everything dies and sanitizes instantly |
| You carry a laptop bag | You carry a credit card |

---

## Hardware — M5Stack Cardputer Zero

| Spec | Value |
|---|---|
| **SoC** | RP3A0 (Pi Zero 2W die), Quad-Core Cortex-A53 |
| **Architecture** | aarch64 / arm64 |
| **RAM** | 512MB LPDDR2 |
| **Display** | 1.9" ST7789V 320×170 RGB565 LCD |
| **Keyboard** | TCA8418 46-key matrix (I2C) |
| **Audio** | ES8390 codec + TPA6130A2 headphone amp (I2S) |
| **IMU** | BMI270 6-axis accelerometer + gyroscope (I2C) |
| **RTC** | RX8130 (I2C) |
| **IO Expander** | PY32IO16 — 16 GPIO + PWM (I2C) |
| **Battery** | BQ27220 fuel gauge (I2C) |
| **WiFi** | 802.11 b/g/n (SDIO) |
| **BT/BLE** | Bluetooth 4.2 + BLE (UART) |
| **IR** | Transceiver (GPIO) |
| **Camera** | IMX219 8MP (CSI) |
| **USB** | USB-C device + USB-A host |
| **Expansion** | Grove (I2C/UART) + 14-pin GPIO header |

Device tree: [`cardputerzero-overlay.dts`](overlays/cardputerzero-overlay.dts) — single comprehensive overlay.

---

## The Constraints We Solved

| Constraint | Our Solution |
|---|---|
| **512MB RAM** | `musl` where possible, `dropbear` over `sshd`, no `postgres`, no heavy daemons. Metasploit excluded. |
| **1.9" 320×170 display** | Textual TUI drill-down launcher — no desktop, 13 categories, 46-key navigation |
| **46-key matrix keyboard** | `Fn` Omni-Key system. Every tool is 2 keypresses from anywhere. |
| **1500mAh battery** | Three power profiles (performance / balanced / stealth). `autosleep`. Radio toggle hotkeys. |
| **No mouse, ever** | `i3` tiling WM backend. tmux splits. Arrow-key everything. |
| **Credit-card size (85×54mm)** | No external dongles needed. IR, BT, WiFi, camera — all on-board. |

---

## Architecture

```
 ┌──────────────────────────────────────────────────────────┐
 │                    ZERO-DAY OS STACK                      │
 ├──────────────────────────────────────────────────────────┤
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │         TEXTUAL TUI  ·  Cyber Launcher           │    │
 │   │                                                   │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │    │
 │   │   │WIFI │ │M5MON│ │ NET │ │  BT │ │  IR │     │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘ └─────┘     │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │    │
 │   │   │ CAM │ │PAYLD│ │RADIO│ │NFC  │ │SHELL│     │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘ └─────┘     │    │
 │   │   ┌─────┐ ┌────────┐ ┌─────┐                     │    │
 │   │   │ SYS │ │OPENCODE│ │OPEN │                     │    │
 │   │   └─────┘ └────────┘ └─────┘                     │    │
 │   │                                                   │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌───────────┐  ┌───────────┐  ┌───────────────────┐   │
 │   │  i3 wm    │  │  OpenCode │  │  Panic System      │   │
 │   │ (tiling)  │  │  (editor) │  │  (kill + wipe)     │   │
 │   └───────────┘  └───────────┘  └───────────────────┘   │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   One-Key Hacking Scripts  /usr/local/bin       │    │
 │   │   wifi-* · net-* · bt-* · ir-* · cam-*          │    │
 │   │   nfc-* · subghz-* · mesh-* · dongle-* · sdr-* │    │
 │   │   payload-craft · revshell-stabilize · panic     │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   Debian Bookworm arm64  +  Kali Rolling repos  │    │
 │   │   aircrack · nmap · bettercap · sqlmap · john    │    │
 │   │   hydra · gobuster · dsniff · responder · curl   │    │
 │   │   hashcat-utils · hcxdumptool · meshtastic       │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   RP3A0 Device Tree Overlays                      │    │
 │   │   SPI (LCD) · I2C (kbd,IMU,battery,RTC,IO)      │    │
 │   │   I2S (audio) · CSI (camera) · GPIO (IR,USB)    │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 └──────────────────────────────────────────────────────────┘
```

---

## Tool Arsenal

Every tool chosen for **sub-100MB RAM at idle**. No fat daemons. No database servers. Metasploit is excluded (requires 1GB+ RAM).

### WiFi Offense
| Command | Description |
|---|---|
| `sudo wifi-scan <iface>` | Quick survey — list all APs, channels, encryption |
| `sudo wifi-deauth <iface> <bssid> <chan>` | Monitor mode + deauth attack |
| `sudo wifi-handshake <iface> <bssid> <chan>` | Capture WPA handshakes → `/opt/cardputer/handshakes/` |
| `sudo wifi-pmkid <iface> <bssid> <chan>` | PMKID capture via hcxdumptool |
| `sudo wifi-evil-twin <ap> <inet> <essid>` | Rogue AP: hostapd + dnsmasq + captive portal |
| `sudo wifi-crack <cap>` | Crack captured handshakes (aircrack/hashcat) |
| `sudo wifi-monitor-toggle` | Toggle managed/monitor mode |

### Network Recon & Attack
| Command | Description |
|---|---|
| `sudo net-discover <iface> [subnet]` | ARP scan + ping sweep |
| `net-quickscan <target> [profile]` | Nmap: quick/web/full/stealth/vuln |
| `sudo net-vulnscan <target>` | Nmap vuln → nikto → whatweb |
| `net-pivot <mode> [args]` | SOCKS5 proxy / chisel tunnel / DNS tunnel |
| `device-classify <nmap_xml>` | Parse nmap XML, classify by OUI and service |
| `threat-intel <ip|cve>` | CVE/CISA KEV lookup via NVD API |

### Bluetooth
| Command | Description |
|---|---|
| `sudo bt-scan` | BLE + Classic discovery |
| `sudo bt-deep <mac>` | Deep enumerate: name, class, LMP, SDP |
| `sudo bt-attack <type> [mac]` | BlueBorne / l2ping flood / RFCOMM scan |
| `sudo ble-gatt <mac>` | GATT service + handle enumeration |

### NFC & Sub-GHz
| Command | Description |
|---|---|
| `sudo nfc-read` | Read NFC tags (Proxmark3 / nfcpy) |
| `sudo nfc-clone <uid\|dump>` | Clone NFC tags |
| `sudo nfc-emulate <type>` | Emulate MIFARE/NTAG/EM4100 |
| `sudo subghz-scan [band]` | Scan Sub-GHz frequencies (RTL-433/CC1101) |
| `sudo subghz-record <freq> <time>` | Record Sub-GHz signals |
| `sudo subghz-replay <file>` | Replay captured Sub-GHz signals |

### Mesh / LoRa
| Command | Description |
|---|---|
| `mesh-chat chat` | Interactive Meshtastic LoRa chat |
| `mesh-setup init` | Initialize Meshtastic node |

### IR — Infrared Hacking
| Command | Description |
|---|---|
| `sudo ir-scan` | Capture and decode IR signals |
| `sudo ir-replay <signal_file>` | Replay captured IR signals |
| `sudo ir-brute <protocol> [device]` | Brute-force IR power codes |

### Camera
| Command | Description |
|---|---|
| `cam-snap [output]` | Capture still image |
| `cam-stream [duration]` | Record video clip |
| `cam-ocr [output]` | Capture + Tesseract OCR |

### Hardware & Radio
| Command | Description |
|---|---|
| `sudo sdr-scan [freq_range]` | RTL-SDR frequency scan |
| `sudo rf-capture [freq]` | Raw RF capture and analysis |
| `sudo gpio-probe` | Enumerate I2C/SPI/UART devices |
| `sudo cardputer-battery` | BQ27220 fuel gauge readout |
| `sudo dongle-setup <cmd>` | RTL8821CU dongle manager |

### Reverse Shells & Payloads
| Command | Description |
|---|---|
| `payload-craft <type> [ip] [port]` | msfvenom wrapper (ARM/x86/Python) |
| `revshell-stabilize` | Cheatsheet for shell stabilization |

### System & Field Ops
| Command | Description |
|---|---|
| `panic` | KILL EVERYTHING — kill processes, wipe history, sanitize |
| `stealth-backlight-toggle` | Kill/restore LCD backlight (stealth mode) |
| `zeroday-boot` | Boot orchestration (drivers, CPU gov, Xorg start) |
| `power-mode <profile>` | performance / balanced / stealth |
| `cardputer-wifi-toggle` | Toggle wlan0 on/off |
| `cardputer-wifi-setup` | Interactive WiFi configurator |
| `usb-gadget-mode <type>` | USB device mode (HID/serial/NCM/storage) |
| `first-boot` | First-boot wizard (filesystem expand, password, WiFi) |
| `opencode-session` | tmux split-screen IDE |
| `tamper-watch` | BMI270 tamper detection daemon |

---

## Keyboard Map — The Omni-Key System

46 keys. One `Fn` key. Zero mouse. Every action is 2 keypresses from anywhere.

```
 ┌─────────────────────────────────────────────┐
 │  Fn + Tab   → Flipper TUI toggle            │
 │  Fn + P     → PANIC (kill all + wipe)       │
 │  Fn + Space → STEALTH (kill backlight)       │
 │  Fn + Return→ Quick terminal                │
 │  Fn + Q     → Close tile                    │
 │  Fn + O     → OpenCode                      │
 │                                              │
 │  Fn + N     → Nmap QuickScan                 │
 │  Fn + B     → Bluetooth scan                │
 │  Fn + S     → Shell listener                │
 │  Fn + W     → WiFi monitor toggle           │
 │  Fn + C     → Camera snap                   │
 │  Fn + I     → IR scan                       │
 │  Fn + A     → opencode-ask                  │
 └─────────────────────────────────────────────┘
```

---

## Building the OS Image

Built from scratch using Docker for full reproducibility. aarch64 cross-compilation via QEMU.

### Prerequisites (x86 Linux Host)

```bash
# Arch Linux / CachyOS
sudo pacman -S docker
sudo systemctl enable --now docker

# Debian / Ubuntu
sudo apt install docker.io
sudo systemctl enable --now docker
```

### Build
```bash
cd pi-gen
chmod +x build-docker.sh
./build-docker.sh
# ~20min. Downloads Debian arm64 base + Kali tools. Go get coffee.
```

Retrieve `.img` from `pi-gen/deploy/` and flash to a **microSD card**:
```bash
sudo dd if=zeroday-os.img of=/dev/sdX bs=4M status=progress conv=fsync
```

### First Boot
1. Login: `root` / `zeroday` — **change immediately**: `passwd`
2. Configure WiFi: `cardputer-wifi-setup`
3. Launch the TUI: `Fn + Tab` or run `cyber_launcher`
4. Open OpenCode: `Fn + O` or run `opencode-session`

---

## Threat Model & Ethics

ZERO-DAY OS is a professional tool for **authorized security testing**. The panic key exists because real pentesters sometimes need to disappear fast. All actions are logged locally for your engagement report.

**Do not use this on networks or devices you don't own or have explicit written authorization to test.**

---

## Credits

- **M5Stack** — Cardputer Zero hardware and official DT overlays
- **Raspberry Pi Foundation** — RP3A0 SoC and pi-gen build system
- **Kali Linux** — Tool repositories
- **OpenCode** — On-device AI-assisted code editor (v1.14.49)
- **dianjixz** — CM0 firmware reference
- **Offensive Security** — Training and tool ecosystem
- **The Flipper Zero community** — TUI design inspiration

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>