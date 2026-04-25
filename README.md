```
 ███████╗███████╗██████╗ ███████╗    ██████╗ ███████╗██████╗ ██╗   ██╗███████╗████████╗
 ██╔════╝██╔════╝██╔══██╗██╔════╝    ██╔══██╗██╔════╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝
 █████╗  █████╗  ██████╔╝███████╗    ██████╔╝█████╗  ██████╔╝██║   ██║███████╗   ██║   
 ██╔══╝  ██╔══╝  ██╔══██╗╚════██║    ██╔══██╗██╔══╝  ██╔═══╝ ██║   ██║╚════██║   ██║   
 ██║     ███████╗██║  ██║███████║    ██║  ██║███████╗██║     ╚█████╔╝███████║   ██║   
 ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝    ╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝ ╚══════╝   ╚═╝   
                                         ██████╗ ██╗   ██╗███████╗██████╗ ████████╗
                                         ██╔══██╗██║   ██║██╔════╝██╔══██╗╚══██╔══╝
                                         ██████╔╝██║   ██║█████╗  ██████╔╝   ██║   
                                         ██╔═══╝ ██║   ██║██╔══╝  ██╔══██╗   ██║   
                                         ██║     ╚█████╔╝███████╗██║  ██║   ██║   
                                         ╚═╝      ╚═════╝ ╚══════╝╚═╝  ╚═╝   ╚═╝   
```

# ZERO-DAY OS

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

ZERO-DAY OS turns the M5Stack Cardputer Zero — a quad-core ARM box with WiFi, BT, Ethernet, IR, a camera, a battery, and a built-in keyboard for $89 — into a pocketable offensive security weapon. Every byte of this distro is optimized for the constraints of 512MB RAM and a 1.9" screen. No desktop. No bloat. No compromises.

**Build status: ✅ Compiles end-to-end — stages 0–5 + export-image produce a bootable 3.5GB SD card image.**

---

## What Makes This Different

You can install Kali on a Raspberry Pi. That's not what this is.

| Stock Pi + Kali | ZERO-DAY OS |
|---|---|
| Boots into a desktop you can't use on 1.9" | Boots straight into a Flipper-style TUI |
| 2GB+ RAM just for the DE | ~60MB idle, 512MB total — 382MB free for tools |
| Mouse required | 46-key Omni-Key system — zero mouse needed |
| Tools are menu items you click | Tools are **2 keystrokes away** from anywhere |
| No hardware awareness | IR, camera, IMU, battery — all weaponized |
| Close lid, pray | Press `Fn + P` — everything dies and sanitizes instantly |
| Single WiFi radio | **Dual WiFi** — built-in + RTL8821CU dongle for simultaneous attack & C2 |
| You carry a laptop bag | You carry a credit card |

---

## The Constraints We Solved

The Cardputer Zero is an incredible machine with brutal constraints. Every design decision in ZERO-DAY OS exists because of one of these:

| Constraint | Our Solution |
|---|---|
| **512MB RAM** | `systemd-resolved` over NetworkManager, no desktop DE, no `postgres`, no heavy daemons. Metasploit excluded (needs 1GB+). |
| **1.9" 320×240 display** | Flipper-style drill-down TUI on i3 tiling backend, X11 fbdev driver, 8pt fonts — no wasted pixels. |
| **46-key matrix keyboard** | `Fn` Omni-Key system. Every tool is 2 keypresses from anywhere. No chording hell. |
| **1500mAh battery** | Three power profiles (performance / balanced / stealth). Radio toggle hotkeys. `rfkill block all` at boot. |
| **No mouse, ever** | `i3` tiling WM backend. tmux splits. Arrow-key everything. |
| **Credit-card size (85×54mm)** | No external dongles needed for basic ops. IR, BT, WiFi, Ethernet, camera — all on-board. |
| **Single WiFi radio** | RTL8821CU dongle on USB-A gives **wlan1** for attacks while **wlan0** stays online for C2 |

---

## Dual-WiFi Architecture

The built-in `brcmfmac` WiFi (`wlan0`) and an RTL8821CU USB dongle (`wlan1`) work **simultaneously** — one for attacks, one for command and control:

```
  ┌─────────────────────────────────────────────────┐
  │              CARDPUTER ZERO                      │
  │                                                 │
  │  wlan0 (built-in brcmfmac)                      │
  │  └── Managed mode ─── C2 / exfil / internet     │
  │      └── ssh, reverse shells, data exfil        │
  │      └── stays connected throughout engagement  │
  │                                                 │
  │  wlan1 (RTL8821CU dongle on USB-A)              │
  │  └── Monitor mode ─── attacks                   │
  │      └── deauth, handshake capture, PMKID        │
  │      └── evil twin, probe sniffing              │
  │      └── can switch to managed for dongle-scan  │
  │                                                 │
  └─────────────────────────────────────────────────┘
```

**Why this matters:** With a single WiFi radio, you must choose between being connected (managed mode) or attacking (monitor mode). With dual-WiFi, you capture handshakes on `wlan1` while staying connected to your team's infrastructure on `wlan0`. No more "fly blind" during attacks.

### dongle-setup

```bash
dongle-setup install    # Build & load RTL8821CU driver (DKMS)
dongle-setup status     # Show dongle + interface status
dongle-setup monitor    # Put dongle into monitor mode (for attacks)
dongle-setup managed    # Put dongle back into managed mode
dongle-setup scan       # Quick WiFi scan using the dongle
dongle-setup test       # Verify dongle is working
```

**Supported dongles:** Any USB WiFi adapter with RTL8821CU chipset (ASUS USB-AC51, TP-Link Archer T2U Nano, etc.). Plug into the USB-A port on the back of the Cardputer Zero. A udev rule ensures it always appears as `wlan1`.

---

## Architecture

```
 ┌──────────────────────────────────────────────────────────┐
 │                    ZERO-DAY OS STACK                      │
 ├──────────────────────────────────────────────────────────┤
 │                                                          │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │           FLIPPER TUI  ·  Textual UI             │    │
 │   │                                                   │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐               │    │
 │   │   │WIFI │ │M5MON│ │ NET │ │  BT │               │    │  ← Level 1: Grid
 │   │   └─────┘ └─────┘ └─────┘ └─────┘               │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐               │    │
 │   │   │ IR  │ │ CAM │ │PAYLD│ │RADIO│               │    │
 │   │   └─────┘ └─────┘ └─────┘ └─────┘               │    │
 │   │   ┌─────┐ ┌─────┐ ┌─────┐                        │    │
 │   │   │SHELL│ │ SYS │ │OPEN │                        │    │
 │   │   └─────┘ └─────┘ └─────┘                        │    │
 │   │                                                   │    │
 │   │        ┌──────────────────────┐                   │    │
 │   │        │  Tool List Scroll    │                   │    │  ← Level 2: Tools
 │   │        └──────────────────────┘                   │    │
 │   │        ┌──────────────────────┐                   │    │
 │   │        │  Guided Actions      │                   │    │  ← Level 3: Presets
 │   │        └──────────────────────┘                   │    │
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
 │   │   revshell-* · payload-* · panic · power-mode  │    │
 │   │   subghz-* · nfc-* · mesh-chat                  │    │
 │   │   dongle-setup · monsterctl · usb-gadget-mode · cam-*   │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   Debian Bookworm minimal + Kali Rolling repos  │    │
 │   │   aircrack · nmap · sqlmap · hashcat · tshark   │    │
 │   │   john · hcxdumptool · nikto · chisel · seclists│    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 │   ┌─────────────────────────────────────────────────┐    │
 │   │   CM0 Device Tree Overlays (PLACEHOLDER pins)   │    │
 │   │   SPI (LCD) · I2C (IMU,battery,RTC) · I2S      │    │
 │   │   CSI (camera) · GPIO (IR,kbd) · SDIO (WiFi)    │    │
 │   │   USB (dongle) · USB-C (gadget mode)            │    │
 │   │   CC1101 (Sub-GHz) · PN532 (NFC) · LoRa (mesh)  │    │
 │   └─────────────────────────────────────────────────┘    │
 │                                                           │
 └──────────────────────────────────────────────────────────┘
```

---

## OpenCode — Your Pocket IDE

OpenCode is a first-class app on ZERO-DAY OS — an AI-assisted code editor and terminal you launch with `Fn + O`. It's not the brain of the distro. It's your workshop inside it.

When you're standing in a parking lot at 2am with a 1.9" screen and no laptop, you need to:
- **Edit attack scripts** on the fly — tweak a bettercap filter, modify a payload before sending
- **Write one-off tools** for targets that don't fit the pre-loaded arsenal
- **Read captured data** — handshakes, scan results, logs — and decide your next move
- **Debug running scripts** with the live console split below your editor

### opencode-session
```bash
opencode-session                        # Full workspace — editor top, console bottom
opencode-session /opt/cardputer r1      # Jump to specific file
opencode-session /path/to/dir filename  # Custom location
```

A tmux split: 70% editor on top, 30% live console on bottom. Workspace lives at `/opt/cardputer/workspace/`.

> **Note:** On armhf, the OpenCode binary is not yet available. A stub is installed that provides the same `opencode-session` tmux workflow with `nano` as a fallback editor. When an armhf binary ships, the stub will be replaced automatically.

### opencode-ask
```bash
opencode-ask "How do I crack a WPA3 handshake?"    # Quick AI prompt
opencode-ask                                         # Interactive — type your question
```

Bound to `Fn + A`. Opens a quick AI prompt without leaving the terminal. Saves questions for later if the AI backend isn't available yet.

### Launching
| Method | How |
|---|---|
| **From the TUI** | Navigate to `[OPEN]` tile on the Flipper grid, press Enter |
| **From anywhere** | `Fn + O` — instant full-screen OpenCode |
| **From terminal** | Run `opencode-session` |

---

## Tool Arsenal

Every tool chosen for **sub-100MB RAM at idle**. No fat daemons. No database servers. Metasploit is excluded — it requires 1GB+ RAM and armhf has 512MB. Use `sqlmap`, `searchsploit`, and manual exploits instead.

### WiFi Offense
| Command | Description |
|---|---|
| `sudo wifi-scan <iface>` | Quick survey — list all APs, channels, encryption |
| `sudo wifi-deauth <iface> [bssid] [chan]` | Monitor mode + deauth attack (use `wlan1` for dual-WiFi) |
| `sudo wifi-handshake <iface> [bssid] [chan]` | Capture WPA handshakes → `/opt/cardputer/handshakes/` |
| `sudo wifi-pmkid <iface> [bssid] [chan]` | PMKID capture via hcxdumptool (no client needed) |
| `sudo wifi-evil-twin <ap> <inet> <essid> [portal]` | Rogue AP + captive portal — harvest WiFi, corporate, or social credentials |
| `sudo wifi-crack <cap>` | Crack captured handshakes (hashcat/aircrack) |
| `sudo wifi-monitor-toggle [iface]` | Switch managed/monitor mode (warns if toggling wlan0 with dongle present) |

### RTL8821CU Dongle
| Command | Description |
|---|---|
| `dongle-setup install` | Build & install RTL8821CU driver (DKMS, first boot) |
| `dongle-setup status` | Show dongle + WiFi interface status (driver, mode, MAC) |
| `dongle-setup monitor` | Put dongle into monitor mode for attacks |
| `dongle-setup managed` | Put dongle back into managed mode |
| `dongle-setup scan` | Quick WiFi scan using the dongle |
| `dongle-setup test` | Verify dongle is working (module, USB, interface) |

### Network Recon
| Command | Description |
|---|---|
| `sudo net-discover <iface> [subnet]` | ARP scan + ping sweep — find every host |
| `net-quickscan <target> [profile]` | Nmap with profiles: `quick` / `web` / `full` / `stealth` / `vuln` |
| `sudo net-vulnscan <target>` | Chain: nmap vuln → nikto → whatweb |
| `net-pivot <mode>` | SOCKS proxy / chisel tunnel / DNS tunnel / ICMP tunnel |
| `ragnar-scan [iface] [quick\|full\|vuln\|stealth]` | Autonomous 3-phase network recon (discover → scan → summarize) |
| `threat-intel cve <CVE-ID>` | NVD CVE lookup |
| `threat-intel search <keyword>` | CISA Known Exploited Vulnerabilities search |
| `threat-intel check <service> [version]` | Check known vulns for a service |
| `threat-intel recent` | Show recent CISA KEV additions |
| `device-classify [nmap_xml]` | Classify network devices from nmap output |

### Bluetooth
| Command | Description |
|---|---|
| `sudo bt-scan` | BLE + Classic discovery — see every device in range |
| `sudo bt-deep <mac>` | Deep enumerate: name, class, SDP, LMP, pairing state |
| `sudo bt-attack <type> [mac]` | BlueBorne / l2ping flood / RFCOMM scan / OBEX push |
| `sudo ble-gatt <mac>` | GATT service + characteristic enumeration, handle dump |

### Reverse Shells & Payloads
| Command | Description |
|---|---|
| `revshell-listen [port]` | Netcat listener with session logging |
| `revshell-gen <type> <ip> [port]` | Generate reverse shell one-liners (bash/python/nc/perl/php/powershell/java/socat/c) |
| `revshell-stabilize` | Shell upgrade cheatsheet — PTY & full TTY |
| `payload-craft <type> <ip> <port> [fmt]` | msfvenom wrapper — ARM/Win/Android payloads |

### IR — Infrared Hacking
| Command | Description |
|---|---|
| `sudo ir-scan` | Capture and decode IR signals from any remote |
| `sudo ir-replay <signal_file>` | Replay captured signals — take over TVs, ACs, projectors |
| `sudo ir-brute <protocol> [device]` | Brute-force IR power codes — turn off every TV in the building |

### Camera — Physical Recon
| Command | Description |
|---|---|
| `cam-snap [output]` | Capture a still image → `/opt/cardputer/loot/cam/` |
| `cam-stream [duration]` | Record a video clip (H.264 @ 1080p30) |
| `cam-ocr [output]` | Capture + Tesseract OCR — read badges, screens, documents on sight |

### Sub-GHz Radio (CC1101 — expansion module)
| Command | Description |
|---|---|
| `sudo subghz-scan [freq_range]` | Scan Sub-GHz frequencies (433/315/868/915MHz) via CC1101, rtl_433, or rfcat |
| `sudo subghz-record <freq> [duration]` | Record Sub-GHz signals to raw capture files |
| `sudo subghz-replay <signal_file> [freq] [repeats]` | Replay captured Sub-GHz signals via CC1101 or YardStick One |

### NFC / RFID (PN532 — expansion module)
| Command | Description |
|---|---|
| `sudo nfc-read [output]` | Read NFC/RFID tags — UID, NDEF, MIFARE Classic dump |
| `sudo nfc-clone <uid_or_dump> [output]` | Clone a tag by UID or from a dump file |
| `sudo nfc-emulate <type_or_uid> [duration]` | Emulate NFC tags — act as a MIFARE, NTAG, or EM4100 |

### Meshtastic — Mesh Networking (LoRa — expansion module)
| Command | Description |
|---|---|
| `mesh-chat install [--port /dev/ttyUSB0]` | Install & configure Meshtastic node |
| `mesh-chat send <node\|channel> <message>` | Send a message to a node or channel |
| `mesh-chat listen [duration]` | Listen for incoming mesh messages |
| `mesh-chat chat [channel]` | Interactive mesh chat (IRC-like) |
| `mesh-chat nodes` | List discovered mesh nodes |
| `mesh-chat info` | Show node info, battery, signal, GPS |
| `mesh-setup install` | Full Meshtastic setup — CLI, wiring, dependencies |
| `mesh-setup init` | Initialize and configure a LoRa node |
| `mesh-setup send <message> [dest]` | Send encrypted message over mesh |
| `mesh-setup listen` | Continuous message monitoring |
| `mesh-setup chat` | Interactive chat mode |
| `mesh-setup relay` | Enable mesh relay / internet bridge |
| `mesh-setup nodes` | List discovered mesh nodes |
| `mesh-setup exfil <file>` | Exfiltrate a file over mesh (chunked base64) |

### Hardware & Radio
| Command | Description |
|---|---|
| `sudo sdr-scan [freq_range] [duration]` | RTL-SDR frequency sweep — heatmap output (default: 433MHz) |
| `sudo rf-capture [freq] [duration] [samplerate]` | Raw IQ capture for offline analysis (default: 433.92MHz, 10s) |
| `sudo gpio-probe` | Enumerate I2C/SPI/UART devices on expansion port |

### M5MonsterC5 — ESP32C5 WiFi Attack Board

The M5MonsterC5 is an ESP32C5-based add-on board running JanOS/projectZero firmware that connects via USB-A or UART serial. It extends ZERO-DAY OS with dedicated WiFi attack hardware — offloading monitor-mode attacks, deauth, evil twin, and sniffing from the Cardputer's own radios.

**`monsterctl` command reference:**

| Subcommand | Description |
|---|---|
| `monsterctl ping` | Ping the board — verify connection (expects `pong`) |
| `monsterctl scan` | Scan for WiFi networks in range |
| `monsterctl select <id> [id...]` | Select target AP(s) by scan index |
| `monsterctl deauth` | Launch deauth attack on selected target |
| `monsterctl evil_twin` | Start evil twin AP on selected target |
| `monsterctl sae_overflow` | WPA3 SAE overflow attack |
| `monsterctl handshake` | Capture WPA/WPA2 handshake |
| `monsterctl sniffer` | Start WiFi packet sniffer |
| `monsterctl blackout` | Mass deauth — disconnect all clients on all visible APs |
| `monsterctl sniffer_dog` | Follow and sniff a specific client's traffic |
| `monsterctl karma` | Karma attack — respond to all probe requests |
| `monsterctl beacon_spam` | Flood beacons with random SSIDs |
| `monsterctl wardrive` | Wardrive scan — log APs with GPS coordinates |
| `monsterctl nmap <target>` | Port scan target via board |
| `monsterctl arp_poison <target>` | ARP poisoning attack |
| `monsterctl rogue_ap` | Standalone rogue AP |
| `monsterctl deauth_detect` | Detect deauth attacks in the area |
| `monsterctl passwords` | Show captured credentials |
| `monsterctl hosts` | Show discovered hosts |
| `monsterctl probes` | Show captured probe requests |
| `monsterctl wifi_connect <ssid> <pass>` | Connect board to a WiFi network |
| `monsterctl wifi_disconnect` | Disconnect board from WiFi |
| `monsterctl gps` | Show GPS coordinates (if GPS module attached) |
| `monsterctl channel_time <ch>` | Set dwell time per channel (ms) |
| `monsterctl list_sd` | List files on board's SD card |
| `monsterctl list_html` | List captive portal HTML pages on board |
| `monsterctl stop` | Stop all running attacks |
| `monsterctl status` | Show board status, connected clients, attack state |
| `monsterctl flash <web\|local\|cardputer>` | Flash firmware to the board |

**Supported attacks:** Deauth, Evil Twin, WPA3 SAE Overflow, Karma, Wardriving, Sniffer, Blackout/SnifferDog, Beacon Spam, Rogue AP, ARP Poisoning, Deauth Detection, Handshake Capture, Nmap

**Board connection:** USB-A or UART at 115200 baud, auto-detected. The `monsterctl` script scans `/dev/ttyUSB*`, `/dev/ttyACM*`, `/dev/ttyAMA0`.

**Firmware flashing:**
```bash
monsterctl flash web          # Flash latest firmware from GitHub
monsterctl flash local        # Flash from local binary
monsterctl flash cardputer    # Flash from cardputer SD card
```

**Links:** [M5MonsterC5-CardputerADV](https://github.com/C5Lab/M5MonsterC5-CardputerADV) · [projectZero](https://github.com/C5Lab/projectZero)

### JanOS Interactive Controller

An interactive TUI front-end for the M5MonsterC5 board. `install-janos` installs, updates, and launches the JanOS-app Python TUI, which provides a menu-driven interface to all M5MonsterC5 attacks without memorizing `monsterctl` commands.

| Command | Description |
|---|---|
| `install-janos` | Install/update/run the JanOS-app interactive TUI |
| `install-janos install` | Clone JanOS-app + install pyserial |
| `install-janos run [/dev/ttyUSB0]` | Launch the interactive TUI (scan, attacks, wardrive, sniffer, evil twin, karma, etc.) |
| `install-janos update` | Pull latest from GitHub |
| `install-janos status` | Check if JanOS-app is installed |

The TUI provides a full interactive menu: **Scan**, **Sniffer**, **Attacks** (Deauth, Blackout, SAE Overflow, Handshaker, Portal, Evil Twin, Beacon Spam, ARP, MITM), **Wardrive + GPS**, and **SD data browser**. Only dependency is `pyserial` (lightweight, <5MB RAM).

**Repository:** [https://github.com/D3h420/JanOS-app](https://github.com/D3h420/JanOS-app)

**Alias:** `monsterctl janos` launches the TUI from the CLI — same as `install-janos run`.

### Ragnar — Autonomous Network Reconnaissance

[Ragnar](https://github.com/PierreGode/Ragnar) is a full Python recon platform requiring 2–8GB RAM — too heavy for the Cardputer Zero's 512MB. ZERO-DAY OS integrates three lightweight scripts inspired by Ragnar's capabilities, each running in <50MB RAM (pure bash + curl + jq):

| Command | Description |
|---|---|
| `ragnar-scan [interface] [quick\|full\|vuln\|stealth]` | Autonomous 3-phase network recon (discover → scan → summarize) |
| `threat-intel cve <CVE-ID>` | NVD CVE lookup |
| `threat-intel search <keyword>` | CISA Known Exploited Vulnerabilities search |
| `threat-intel check <service> [version]` | Check known vulns for a service |
| `threat-intel recent` | Show recent CISA KEV additions |
| `device-classify [nmap_xml]` | Classify network devices from nmap output (vendor OUI + service fingerprinting) |

For the full Ragnar experience (AI analysis, Nuclei, ZAP, traffic analysis, web dashboard), run it on a separate machine with 8GB+ RAM and use ZERO-DAY OS as the hands-on attack tool.

### System & Field Ops
| Command | Description |
|---|---|
| `panic` | KILL EVERYTHING — wipe history, kill processes, sanitize |
| `cardputer-wifi-toggle` | Toggle `wlan0` on/off (saves battery, goes dark) |
| `dongle-setup status` | Show dual-WiFi interface status |
| `power-mode <profile>` | Switch power profile (see below) |
| `cardputer-wifi-setup` | Interactive WiFi configurator |
| `cardputer-battery` | Read BQ27220 fuel gauge — voltage, %, time remaining |

### USB Gadget Mode — The Silent Vector
| Command | Description |
|---|---|
| `sudo usb-gadget-mode hid` | USB keyboard (Rubber Ducky mode) — execute payloads on victim PC |
| `sudo usb-gadget-mode serial` | USB serial console (debug / serial bridge) |
| `sudo usb-gadget-mode ncm` | USB network adapter (share internet with host) |
| `sudo usb-gadget-mode mass` | USB mass storage (exfiltrate data from loot directory) |
| `sudo usb-gadget-mode off` | Disable gadget mode, return to host |

---

## Keyboard Map — The Omni-Key System

46 keys. One `Fn` key. Zero mouse. Every action is 2 keypresses from anywhere.

```
 ┌─────────────────────────────────────────────┐
 │  Fn + Tab   → Flipper TUI toggle            │
 │  Fn + P     → PANIC (kill all + wipe)       │
 │  Fn + Space → STEALTH (kill backlight)      │
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
 │  Fn + D     → Dongle status                 │
 │  Fn + A     → opencode-ask                  │
 └─────────────────────────────────────────────┘
```

Inside the Flipper TUI:
| Key | Action |
|---|---|
| `↑ ↓ ← →` | Navigate grid and lists |
| `Enter` | Drill into category or execute action |
| `Esc` | Go back one level / exit |

---

## Loot Directory

Every captured artifact, organized:

```
/opt/cardputer/
├── handshakes/         # WPA .cap files — ready for hashcat
├── pmkid/              # PMKID hashes (hashcat mode 22000)
├── payloads/           # Generated payloads (msfvenom output)
├── workspace/          # OpenCode working directory
├── loot/
│   ├── recon/          # Nmap XML/text, nikto reports
│   ├── bt/             # Bluetooth enumeration dumps
│   ├── ble/            # BLE GATT handle dumps
│   ├── ir/             # Captured IR signals (replay-ready)
│   ├── cam/            # Camera captures + OCR output
│   ├── rf/             # SDR/RF captures, Sub-GHz signal recordings
│   ├── nfc/            # NFC tag dumps (UID, NDEF, MIFARE .mfd files)
│   ├── captive/        # Captive portal credential logs
│   └── screenshot/     # Scrot framebuffer grabs
└── config/
    ├── attack-profiles/    # Saved TUI presets (target profiles)
    ├── wordlists/          # Seclists + custom wordlists
    └── ir_codes/           # IR signal database for ir-brute
```

---

## Hardware — Fully Weaponized

Every sensor and radio on the Cardputer Zero is mapped and ready:

| Hardware | Interface | Offensive Use |
|---|---|---|
| **1.9" LCD (ST7789v3)** | SPI | Flipper-style TUI — 3-level drill-down, no wasted pixels |
| **46-key matrix** | I2C + GPIO | Omni-Key system — every tool 2 keys away |
| **WiFi (802.11 b/g/n)** | SDIO | Monitor mode, deauth, handshakes, evil twin (`wlan0`) |
| **RTL8821CU dongle** | USB-A | **Dual-WiFi** — `wlan1` for attacks, `wlan0` for C2 |
| **Bluetooth 4.2 / BLE** | UART | Device enumeration, BlueBorne, GATT exploration |
| **10/100 Ethernet** | RMII | Wired network access, ARP spoofing, pivoting |
| **IR Transceiver** | GPIO | Capture & replay remotes, brute-force power codes |
| **IMX219 Camera (8MP)** | CSI (4-lane) | Physical recon, badge OCR, surveillance |
| **BMI270 IMU** | I2C | Tamper detection — auto-lock/auto-wipe on movement |
| **BQ27220 Fuel Gauge** | I2C | Real-time battery %, voltage, estimated runtime |
| **ES8389 Audio Codec** | I2S | MEMS mic for audio capture, 1W speaker for alerts |
| **RX8130CE RTC** | I2C | Hardware clock — accurate timestamps across reboots |
| **USB-A Host** | USB 2.0 | RTL8821CU dongle, Rubber Ducky, Bash Bunny, RTL-SDR |
| **USB-C Host** | USB 2.0 | Same as USB-A — dual USB host |
| **USB-C Device** | USB 2.0 | Plug into victim PC → reverse shell / HID attack |
| **Expansion Port** | HY2.0-4P + 2.54-14P | GPIO, SPI, I2C, UART — CC1101 Sub-GHz, PN532 NFC, Meshtastic LoRa |
| **CC1101 Sub-GHz** | SPI (2.54-14P) | 315/433/868/915MHz transceiver — garage doors, weather stations, key fobs |
| **PN532 NFC/RFID** | I2C/UART (Grove) | Read, clone, and emulate MIFARE/NTAG/EM4100 tags |
| **Meshtastic LoRa** | UART (Grove) | Encrypted mesh networking — off-grid comms, data exfiltration |

> **Note:** DTS overlay GPIO/I2C pin assignments are PLACEHOLDER — hardware not yet shipped. Actual pinout will be updated when the Cardputer Zero arrives.

### Power Profiles
```bash
sudo power-mode performance   # 1GHz quad, all radios, all cores — ~4hr battery
sudo power-mode balanced      # 800MHz, BT off, WiFi on — ~6hr battery
sudo power-mode stealth       # 600MHz single core, WiFi off, IR off, screen dim — ~10hr battery
```

---

## Panic System — The Kill Switch

This isn't just `kill -9`. The panic key (`Fn + P`) is designed for the moment someone looks over your shoulder:

**What happens in 0.3 seconds:**
1. `kill -9` every offensive process (aircrack, bettercap, nmap, msfconsole, kismet, all shells)
2. Wipe `~/.bash_history`, `/tmp/*`, tmux session history
3. Clear screen buffer — terminal shows only a login prompt
4. Log the panic event with timestamp to `/opt/cardputer/panic.log`

**Then press `Fn + Space` for STEALTH mode:**
- Backlight killed — device appears powered off
- WiFi radio off — no RF emissions
- Any key press wakes the screen, no evidence remains

---

## Building the OS Image

Built from scratch using Docker for full reproducibility. The entire build takes ~13 minutes on a modern x86 machine.

### Prerequisites (x86 Linux Host)

ARM emulation required to build on x86:

**Arch Linux / CachyOS:**
```bash
sudo pacman -S qemu-user-static binfmt-support
sudo systemctl enable --now systemd-binfmt
```

**Debian / Ubuntu:**
```bash
sudo apt install qemu-user-static binfmt-support
```

### Quick Build
```bash
cd pi-gen
./build-docker.sh
# ~13 min. Downloads Debian base + packages. Go get coffee.
```

The output lands in `pi-gen/deploy/` as a `.zip` file containing the raw `.img`.

### Flash to microSD
```bash
unzip pi-gen/deploy/2026-04-25-zeroday-os-.zip
sudo dd if=pi-gen/work/zeroday-os/export-image/image-zeroday-os-.img \
     of=/dev/sdX bs=4M status=progress conv=fsync
```

Or use [BalenaEtcher](https://etcher.balena.io/). Use a **32GB+ microSD** card — the image is 3.5GB raw but you'll want room for captures, wordlists, and workspace.

### Build Architecture

The build follows pi-gen conventions — numbered stages with `00-packages*` and `01-run.sh` files:

| Stage | Purpose | Key Contents |
|---|---|---|
| **stage0** | Bootstrap | Direct debootstrap call (bypasses `bootstrap()` due to GPG key issues in Docker) |
| **stage1** | Base system | Firmware, kernel packages, base tools |
| **stage2** | Networking | `systemd-networkd`, `systemd-resolved`, network configs, `rfkill block all` at boot |
| **stage3** | ZERO-DAY core | Hostname, root password, locale, X11+fbdev, i3 WM, hardware enable, TUI, boot scripts |
| **stage4** | Hacking tools | WiFi tools, network tools, BT, exploits (no metasploit), reverse shells, IR, camera, SDR, wordlists, dongle, Sub-GHz, NFC, Meshtastic |
| **stage5** | Finalize | First-boot service, OpenCode, cleanup |
| **export-image** | Image creation | 3.5GB image: 508M FAT32 boot + 3G ext4 root, PARTUUID fstab, zip compression |

### Build Bugs Fixed (for fork maintainers)

These were the hard-won lessons from getting the build to compile end-to-end:

| Bug | Root Cause | Fix |
|---|---|---|
| DNS failure in chroot at stage3+ | `stage2/02-net-tweaks` symlinked `/etc/resolv.conf` to systemd-resolved stub that doesn't exist in chroot | Removed symlink from stage2; added to first-boot script instead; `on_chroot()` now validates resolv.conf before every chroot |
| `chpasswd` fails in chroot | PAM is non-functional in chroot environment | Use `openssl passwd -6` on host, then `sed` the hash directly into `/etc/shadow` |
| `$PACKAGES` heredoc expansion | Multi-line variable in heredoc caused each line to be treated as a separate command | Pipe through `tr '\n' ' '` in `build.sh` |
| `losetup -f` returns garbage in Docker | Returns `/dev/loop0 (lost)` instead of `/dev/loop0` | Strip after space; add `modprobe loop`; fallback to `mknod` |
| `export-image` missing work directory | `STAGE_WORK_DIR` doesn't exist when `prerun.sh` runs | Added `mkdir -p "${STAGE_WORK_DIR}"` |
| `01-set-partuuid` chroots into nonexistent rootfs | After image is packed, no rootfs to chroot into | Made it a conditional no-op |

---

## First Boot

1. Insert microSD, power on the Cardputer Zero
2. Login: `root` / `zeroday` — **change immediately**: `passwd`
3. First-boot service runs automatically: expands filesystem, generates SSH keys, configures systemd-resolved DNS
4. Configure WiFi: `cardputer-wifi-setup`
5. (Optional) Plug in RTL8821CU dongle → `dongle-setup install`
6. Launch the TUI: `Fn + Tab` or run `cyber_launcher`
7. Open OpenCode: `Fn + O` or run `opencode-session`

---

## Field Workflows

Real scenarios. Real keypresses.

### Walk into a building, own the WiFi (with dongle)
```
Fn + D               → Dongle status (confirm wlan1 is present)
dongle-setup install  → Build driver (first time only)
dongle-setup monitor  → wlan1 in monitor mode
                       → wlan0 stays online for C2, exfil, reverse shells

wifi-handshake wlan1 <bssid> <chan>
Fn + P                → PANIC — both radios killed, all traces wiped
```

### Evil twin + captive portal (credential harvesting)
```
wifi-evil-twin wlan0 eth0 "CorporateWiFi" corporate
                               │
              ┌────────────────┤
              │  Creates rogue AP: "CorporateWiFi"
              │  Captive portal: fake VPN login page
              │  DNS hijack: all queries → portal
              │  NAT relay: internet works after "login"
              │
              │  [+] CAPTURED: jsmith:Winter2024! from 10.0.0.2
              │  [+] CAPTURED: admin:CompanyP@ss from 10.0.0.3
              └────────────────┘
```

### NFC cloning (with PN532 on Grove port)
```
nfc-read                       → Scan tag: UID AA:BB:CC:DD, MIFARE Classic 1K
nfc-clone AA:BB:CC:DD          → Write UID to blank tag
nfc-emulate mifare             → Emulate a MIFARE tag (act as a card)
```

### Plug into victim's PC (USB gadget mode)
```
sudo usb-gadget-mode hid    → Cardputer becomes a keyboard
sudo usb-gadget-mode mass   → Cardputer becomes a USB drive
sudo usb-gadget-mode ncm    → Cardputer becomes a network adapter
```

### Get caught? Vanish.
```
Fn + P             → PANIC
                   → Kills: aircrack, bettercap, nmap, all shells
                   → Wipes: bash history, tmux sessions, /tmp/*
                   → Clears: screen buffer
Fn + Space          → STEALTH (backlight off, device looks powered down)
```

---

## Threat Model & Ethics

ZERO-DAY OS is a professional tool for **authorized security testing**. The panic key exists because real pentesters sometimes need to disappear fast. All actions are logged locally for your engagement report.

**Do not use this on networks or devices you don't own or have explicit written authorization to test.**

---

## Quick Reference Card

```
monsterctl status    # M5MonsterC5 board status
monsterctl scan      # M5MonsterC5 WiFi scan
install-janos run    # JanOS interactive TUI
ragnar-scan eth0 quick # Autonomous network recon
```

---

## Credits

- **M5Stack** — Cardputer Zero hardware
- **Raspberry Pi Foundation** — CM0 and pi-gen
- **Kali Linux** — Tool repositories
- **OpenCode** — On-device AI-assisted code editor
- **morrownr** — RTL8821CU Linux driver
- **Offensive Security** — Training and tool ecosystem
- **The Flipper Zero community** — TUI design inspiration
- **Meshtastic** — Encrypted mesh networking protocol

---

<p align="center">
<strong>Built for the field. Designed for the edge. Fits in your wallet.</strong>
</p>