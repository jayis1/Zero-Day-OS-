# ZERO-DAY OS v4.5.0

**The first penetration testing OS built for a credit-card-sized computer you can hold in one hand.**

Built for M5Stack Cardputer Zero — quad-core ARM64, 512MB RAM, 1.9" LCD, 46-key keyboard, WiFi, BT, IR, camera, battery.

---

## What's New in v4.5.0 — Hacker Betterments + Raspyjack Ports

This release adds a full offensive tool upgrade focused on real-world pentest workflows.  Every tool is designed for the Cardputer Zero's constraints: arm64 native, starts on-demand, exits cleanly, saves to `/opt/cardputer/loot/`, and never runs in the background unless you start it.

---

### WiFi Offense Upgrades

#### `wps-attack` — WPS Pixie Dust + PIN Brute-Force
Attack WPS-enabled APs without needing any clients connected.

- **Phase 1: Pixie Dust** (offline, 90s) — `reaver -K 1` against the target
- **Phase 2: Online PIN brute-force** — with configurable delay and lockout avoidance
- Scans for WPS APs first with `wash`, shows lock status before attacking
- Cracked PIN + PSK saved to `/opt/cardputer/loot/creds/wps_creds.txt`
- Deps: `apt install reaver` (arm64 native on Kali)

#### `wifi-probe-dump` — Passive MAC→SSID Probe Map
See exactly which devices are nearby and what networks they're looking for.

- Puts dongle into monitor mode, hops channels 1-13
- Captures `Probe Request` frames with `tshark` (or `tcpdump` fallback)
- Builds live `MAC → [SSIDs searched]` table in real-time
- Saves JSON: `/opt/cardputer/loot/wifi/probes_<timestamp>.json`
- Output feeds directly into `wifi-karma-ap`

#### `wifi-karma-ap` — KARMA Attack / Rogue AP
Automatically clone the most-probed SSID to lure devices to your AP.

- Reads probe dump JSON, picks the highest-probed SSID automatically
- Spins up `hostapd` + `dnsmasq` as an open AP with that SSID
- All DNS resolves → your IP (ready for captive portal)
- Pairs with existing `captive-portal` for credential harvesting
- Or pass SSID manually: `wifi-karma-ap wlan1 wlan0 "MyNetwork"`

#### `wifi-autocrack` — 4-Tier Auto-Crack (Simplified)
Hand it a `.cap` file and walk away.

- **Tier 1**: 50 common router default passwords (instant, no wordlist)
- **Tier 2**: `rockyou.txt` wordlist (5min cap)
- **Tier 3**: `rockyou` + hashcat rules `best64` + `d3ad0ne`
- **Tier 4**: 8-digit mask (`?d×8` and `?l?l?l?l?d?d?d?d`)
- Converts `.cap → .hc22000` via `hcxpcapngtool`, falls back to `aircrack-ng`
- No daemon, no watching — just `wifi-autocrack <file>`

---

### Network Attack Upgrades

#### `dhcp-attack` — DHCP Starvation + Passive Snoop
Two modes, one tool:

- **`starve`**: Flood DHCP server with random MACs to exhaust its lease pool — no clients can connect
- **`snoop`**: Passively capture every DHCP client hostname, IP, gateway, and DNS server on the network
- Uses `python3-scapy` (arm64 apt native, no RPi.GPIO)

#### `vlan-hop` — VLAN Hopping
Reach VLANs you shouldn't have access to:

- **`detect`**: Passive 802.1Q tag sniffing — see which VLANs exist
- **`double-tag`**: Send double-tagged frames (outer=native, inner=target) to hop VLANs
- **`dtp`**: Send DTP trunk negotiation frames to make the switch think you're a trunk port
- Uses `python3-scapy` inline, no external dep install at runtime

#### `snmp-scan` — SNMP Community Brute + MIB Walk
Find SNMP devices and extract everything from them:

- Discovers SNMP hosts on subnet via `nmap -sU -p 161` or direct probe
- Brutes 20 common community strings (`public`, `private`, `cisco`, etc.)
- Walks: `sysDescr`, `sysName`, `sysLocation`, `ifTable`, `ipRouteTable`, `hrStorageTable`
- Deps: `apt install snmp snmp-mibs-downloader` (arm64 native)

#### `http-probe` — HTTP Service Prober
Rapid web service discovery and path enumeration:

- Probes default web ports on a host or auto-discovers from last nmap XML
- Checks 90+ juicy paths: admin panels, `.git`, `.env`, `/actuator`, API docs, DB admin UIs, backup files
- Flags CMS, DevOps dashboards, DB admin UIs from banner responses
- `http-probe auto` reads your last nmap scan automatically

---

### Post-Exploitation Upgrades

#### `kerberoast` — Kerberoasting + AS-REP Roasting
Extract crackable hashes from Active Directory:

- **Kerberoast**: Request TGS hashes for SPN accounts (hashcat mode 13100)
- **AS-REP Roast**: Extract AS-REP hashes from accounts without pre-auth (hashcat mode 18200)
- Uses `impacket` (`python3-impacket` on Debian bookworm arm64)
- Hashes ready for `hashcat` offline cracking

#### `pass-spray` — Lockout-Safe Password Sprayer
SSH, FTP, SMB, HTTP-basic, HTTP-form — all with lockout protection:

- **Spray mode**: 1 password across all users before trying the next (minimises lockout)
- Configurable delay + jitter between attempts
- Per-user fail counter stops spraying locked accounts
- `pass-spray auto` discovers services from last nmap scan and sprays all of them

#### `privesc-check` — Local Privilege Escalation Scanner
Run post-compromise to find your path to root:

- SUID/SGID binaries (checks against GTFOBins)
- `sudo -l` misconfigs + GTFOBins flags
- File capabilities (`cap_setuid`, `cap_sys_admin`, etc.)
- Writable cron scripts, world-writable PATH dirs
- Readable `/etc/shadow`, NFS `no_root_squash`
- Docker socket, kernel version CVE hints

#### `post-harvest` — Credential Harvester
Grab everything useful from a compromised host:

- Chrome/Firefox login DBs, SSH private keys + configs + known_hosts
- Shell history credential patterns, `/proc/*/environ` secrets
- WordPress configs, `.env` files, database YAMLs
- AWS/GCP/Azure/k8s/Docker registry credentials

#### `engagement-clean` — Forensic Artifact Wiper
Clean up your traces after an engagement. **Your loot is always protected.**

- `quick`: shell history, temp files, SSH known_hosts
- `full`: everything above + auth.log, syslog, wtmp/btmp, journald vacuum
- `select`: interactive — pick exactly what to wipe
- **Never touches `/opt/cardputer/loot/`** — your captured data is always safe

---

### Wordlists

#### `wifi_top4800.txt`
4,800-password WiFi-specific wordlist from Raspyjack — real-world passwords found in WiFi audits. Installed to `/opt/cardputer/config/wordlists/wifi_top4800.txt`. Used automatically by `wifi-autocrack` Tier 1 and `pass-spray common`.

---

### Launcher + Keybindings

All new tools are accessible from the `cyber_launcher` GUI:

| Tile | New Tools |
|---|---|
| WIFI | WPS Pixie Dust, Probe Dump, Karma AP, Crack→wifi-autocrack |
| NET | VLAN Hop, DHCP Starve, DHCP Snoop, SNMP Scan, HTTP Probe |
| PAYLD | Kerberoast, Engagement Clean, Pass Spray, Privesc Check, Post Harvest |
| SYS | Engagement Clean |

New compositor Fn-key bindings:
- `Fn+K` → `http-probe auto`
- `Fn+V` → `privesc-check quick`
- `Fn+E` → `post-harvest quick`

---

### pi-gen: stage4/17-raspyjack-ports

New build stage installs all tools + arm64-compatible apt dependencies:

```
reaver              — WPS attack (arm64 Kali)
tshark / tcpdump    — packet capture (arm64 native)
python3-scapy       — DHCP/VLAN attacks (arm64 apt, no RPi.GPIO)
snmp snmp-mibs-downloader  — SNMP walks (arm64 native)
python3-impacket    — Kerberoasting (arm64 Debian bookworm)
smbclient           — SMB spraying (arm64 native)
sshpass             — SSH spraying (arm64 native)
hashcat             — WiFi crack tiers 2-4 (CPU-only arm64)
hcxtools            — .cap → .hc22000 conversion
```

All installs are best-effort (`2>/dev/null || echo deferred`) — missing Kali arm64 packages won't break the image build.

---

## Also Includes (carried forward from v4.3.x)

- `zeroday-comp` — Smithay 0.7 Wayland compositor, full protocol support, ~1.5MB
- `zeroday-term` — Rust terminal emulator, 320×170 optimized, status bar, ~1.2MB
- `zeroday-fm` — Rust TUI file explorer, 46-key optimized, ~1.9MB
- `zeroday-trail` — WiFi fingerprint breadcrumb navigation, ~1.1MB
- Jellyfin Media Player (Qt5, built from source, 1080P HDMI)
- HDMI dual-screen: LCD = controls, HDMI = content
- BLE Remote API (Flipper Zero-style GATT server)
- M5MonsterC5 ESP32C5 WiFi attack board support
- GPS, OLED, NFC, Sub-GHz, IR, SDR, Meshtastic LoRa

---

## Hardware

M5Stack Cardputer Zero — Quad-Core ARM Cortex-A53, 512MB LPDDR2, 1.9" LCD 320×170, 46-key matrix keyboard, WiFi 802.11 b/g/n, BT 4.2+BLE, IR transceiver, IMX219 8MP camera, 1500mAh battery, USB-C + USB-A host, Grove + ExtPort expansion.

---

*Built for the field. Designed for the edge. Fits in your wallet.*
