#!/usr/bin/env python3
"""
ZERO-DAY OS - Pygame GUI Launcher
Target: M5Stack Cardputer Zero (320x170, 46-key, no HW acceleration)
"""

import os
import sys
import math
import re
import shlex
import signal
import socket
import threading
import select
import subprocess
import glob
import random

# Pygame fallback to x11
os.environ.setdefault("SDL_VIDEODRIVER", "x11")

try:
    import pygame
except ImportError:
    print("Pygame required: pip install pygame")
    sys.exit(1)

# ==========================================
# CONFIGURATION & DATA MODEL
# ==========================================
SCREEN_W, SCREEN_H = 320, 170
FPS_TARGET = 30
WT_PORT = 42420
WT_MAX_TX_SECS = 30

# Danish radio stream URLs (HLS/AAC streams for ffplay)
WT_RADIO_URLS = {
    "DR_P1":  "https://drradio1-lh.akamaihd.net/i/p1_9@143503/master.m3u8",
    "DR_P3":  "https://drradio3-lh.akamaihd.net/i/p3_9@143506/master.m3u8",
    "NOVA":   "https://stream.bauermedia.fi/nova/nova_64.aac",
    "POPFM":  "https://stream.bauermedia.fi/popfm/popfm_64.aac",
}

# Colors - Kali Nethunter Theme
BG_COLOR = (5, 8, 15)        # Deep abyss blue-black
DIM_BG = (15, 20, 35)        # Dark panel background
TEXT_PRIMARY = (43, 204, 255)# Kali Cyan
TEXT_SECONDARY = (100, 130, 160)
TEXT_WHITE = (240, 250, 255)
CMD_TEXT = (255, 255, 255)
CMD_FLAG = (255, 75, 75)     # Kali Red
CMD_PATH = (43, 204, 255)

CATEGORIES = [
    {"name": "WIFI",      "key": "WIFI",     "color": (43, 204, 255)},  # Cyan
    {"name": "M5MONSTER", "key": "M5MONSTER","color": (255, 75, 75)},   # Kali Red
    {"name": "NET",       "key": "NET",      "color": (0, 150, 255)},   # Deep Blue
    {"name": "BT",        "key": "BT",       "color": (100, 150, 255)}, # Soft Blue
    {"name": "IR",        "key": "IR",       "color": (255, 120, 50)},  # Orange
    {"name": "CAM",       "key": "CAM",      "color": (255, 50, 150)},  # Pink
    {"name": "PAYLD",     "key": "PAYLD",    "color": (255, 200, 50)},  # Gold
    {"name": "RADIO",     "key": "RADIO",    "color": (180, 50, 255)},  # Purple
    {"name": "MEDIA",     "key": "MEDIA",    "color": (50, 255, 100)},  # Green
    {"name": "SHELL",     "key": "SHELL",    "color": (255, 75, 75)},   # Red
    {"name": "SYS",       "key": "SYS",      "color": (150, 160, 170)}, # Grey-Blue
    {"name": "OPENCODE",  "key": "OPENCODE", "color": (255, 255, 0)},   # Yellow
    {"name": "OPEN",      "key": "OPEN",     "color": (43, 204, 255)},  # Cyan
]

TOOLS = {
    "WIFI": [
        {"name": "Scan Networks",     "desc": "Quick WiFi survey (wlan0)",       "cmd": "sudo wifi-scan",                    "need_root": True},
        {"name": "Survey Logger",     "desc": "Continuous WiFi logging",         "cmd": "sudo wifi-survey-log",              "need_root": True},
        {"name": "Capture Handshake", "desc": "WPA handshake capture",           "cmd": "sudo wifi-handshake wlan1",         "need_root": True,  "args": ["BSSID", "CHANNEL"]},
        {"name": "PMKID Capture",     "desc": "PMKID clientless capture",        "cmd": "sudo wifi-pmkid wlan1",             "need_root": True,  "args": ["BSSID", "CHANNEL"]},
        {"name": "Deauth Attack",     "desc": "Deauthenticate target AP",        "cmd": "sudo wifi-deauth wlan1",            "need_root": True,  "args": ["BSSID", "CHANNEL"]},
        {"name": "Evil Twin",         "desc": "Rogue AP with captive portal",    "cmd": "sudo wifi-evil-twin",               "need_root": True,  "args": ["AP_IFACE", "INET_IFACE", "ESSID"]},
        {"name": "Crack Handshake",   "desc": "Crack .cap with john",            "cmd": "wifi-crack",                        "need_root": False, "args": ["CAP_FILE"]},
        {"name": "Monitor Toggle",    "desc": "Toggle managed/monitor mode",     "cmd": "sudo wifi-monitor-toggle",          "need_root": True},
        {"name": "MAC Rotate",        "desc": "Randomize MAC address",           "cmd": "sudo mac-rotate wlan0",             "need_root": True},
        {"name": "Dongle Monitor",    "desc": "RTL8821CU → monitor mode",        "cmd": "sudo dongle-setup monitor",         "need_root": True},
        {"name": "Dongle Managed",    "desc": "RTL8821CU → managed mode",        "cmd": "sudo dongle-setup managed",         "need_root": True},
    ],
    "M5MONSTER": [
        {"name": "JanOS TUI",         "desc": "Full interactive controller",     "cmd": "install-janos run",                 "need_root": False},
        {"name": "Detect Board",      "desc": "Ping MonsterC5 board",            "cmd": "monsterctl status",                 "need_root": False},
        {"name": "Scan Networks",     "desc": "WiFi scan via ESP32C5",           "cmd": "sudo monsterctl scan",              "need_root": True},
        {"name": "Deauth Attack",     "desc": "Deauthenticate selected APs",     "cmd": "sudo monsterctl deauth",            "need_root": True},
        {"name": "Evil Twin",         "desc": "Rogue AP + captive portal",       "cmd": "sudo monsterctl evil_twin",         "need_root": True},
        {"name": "WPA3 SAE Overflow", "desc": "SAE commit flood DoS attack",     "cmd": "sudo monsterctl sae_overflow",      "need_root": True},
        {"name": "Handshake Capture", "desc": "Capture 4-way handshake",         "cmd": "sudo monsterctl handshake",         "need_root": True},
        {"name": "Karma Attack",      "desc": "Respond to probe requests",       "cmd": "sudo monsterctl karma",             "need_root": True,  "args": ["PROBE_N"]},
        {"name": "Wardrive",          "desc": "GPS-tagged WiFi scan",            "cmd": "sudo monsterctl wardrive",          "need_root": True},
        {"name": "Sniffer",           "desc": "WiFi packet sniffer",             "cmd": "sudo monsterctl sniffer",           "need_root": True},
        {"name": "Stop Attack",       "desc": "Stop current MonsterC5 attack",   "cmd": "monsterctl stop",                   "need_root": False},
        {"name": "Install JanOS-app", "desc": "Install interactive TUI",         "cmd": "install-janos install",             "need_root": True},
    ],
    "NET": [
        {"name": "Discover Hosts",    "desc": "ARP scan + ping sweep",           "cmd": "sudo net-discover eth0",            "need_root": True,  "args": ["SUBNET"]},
        {"name": "Quick Scan",        "desc": "Nmap top 100 ports",              "cmd": "net-quickscan",                     "need_root": False, "args": ["TARGET"]},
        {"name": "Web Scan",          "desc": "Nmap web ports + scripts",        "cmd": "net-quickscan",                     "need_root": False, "args": ["TARGET"], "extra": "web"},
        {"name": "Vuln Scan",         "desc": "Nmap vuln + nikto + whatweb",     "cmd": "sudo net-vulnscan",                 "need_root": True,  "args": ["TARGET"]},
        {"name": "Full Scan",         "desc": "All 65535 ports + scripts",       "cmd": "net-quickscan",                     "need_root": False, "args": ["TARGET"], "extra": "full"},
        {"name": "IoT Scan",          "desc": "IoT protocol discovery",           "cmd": "iot-scan",                          "need_root": False, "args": ["TARGET"]},
        {"name": "Gobuster",          "desc": "Directory/DNS brute-forcer",       "cmd": "gobuster",                          "need_root": False},
        {"name": "Pivot (SOCKS)",     "desc": "SOCKS proxy via SSH",             "cmd": "tunnel-mgr socks",                   "need_root": False, "args": ["TARGET"]},
        {"name": "Pivot (Forward)",   "desc": "Local port forward",              "cmd": "tunnel-mgr forward",                 "need_root": False},
        {"name": "C2 Listener",       "desc": "Encrypted C2 shell (socat)",      "cmd": "quick-c2 listen",                   "need_root": False, "args": ["PORT"]},
        {"name": "C2 Payloads",       "desc": "Generate shell one-liners",       "cmd": "quick-c2 payload",                  "need_root": False},
        {"name": "DoH Proxy",         "desc": "DNS-over-HTTPS tunnel",            "cmd": "doh-proxy start",                    "need_root": True},
        {"name": "ARP Spoof",         "desc": "MITM via arp spoofing",            "cmd": "sudo arpspoof",                      "need_root": True,  "args": ["TARGET"]},
    ],
    "BT": [
        {"name": "Scan Devices",      "desc": "BLE + Classic discovery",         "cmd": "sudo bt-scan",                      "need_root": True},
        {"name": "Deep Enumerate",    "desc": "Name, class, SDP, LMP",           "cmd": "sudo bt-deep",                      "need_root": True,  "args": ["MAC"]},
        {"name": "BlueBorne Test",    "desc": "Test BlueBorne vulnerability",    "cmd": "sudo bt-attack blueborne",          "need_root": True,  "args": ["MAC"]},
        {"name": "L2Ping Flood",      "desc": "L2CAP ping flood (DoS)",          "cmd": "sudo bt-attack l2ping_flood",       "need_root": True,  "args": ["MAC"]},
        {"name": "RFCOMM Scan",       "desc": "Scan RFCOMM channels",            "cmd": "sudo bt-attack rfcomm_scan",        "need_root": True,  "args": ["MAC"]},
        {"name": "GATT Enumerate",    "desc": "BLE services + handles",          "cmd": "sudo ble-gatt",                     "need_root": True,  "args": ["MAC"]},
        {"name": "Bettercap",         "desc": "MITM + BLE attack framework",     "cmd": "sudo bettercap",                    "need_root": True},
    ],
    "IR": [
        {"name": "Scan IR Signal",    "desc": "Capture remote control signals",  "cmd": "sudo ir-scan",                      "need_root": True},
        {"name": "Replay Signal",     "desc": "Replay captured IR signal",       "cmd": "sudo ir-replay",                    "need_root": True,  "args": ["FILE"]},
        {"name": "Brute Force TV",    "desc": "TV power codes (NEC protocol)",   "cmd": "sudo ir-brute nec tv",              "need_root": True},
        {"name": "Brute Force AC",    "desc": "AC power codes brute force",      "cmd": "sudo ir-brute nec ac",              "need_root": True},
    ],
    "CAM": [
        {"name": "Snap Photo",        "desc": "Capture still image from camera", "cmd": "cam-snap",                          "need_root": False},
        {"name": "Record Video",      "desc": "Record video clip",               "cmd": "cam-stream",                        "need_root": False, "args": ["DURATION"]},
        {"name": "OCR Capture",       "desc": "Photo + text recognition",        "cmd": "cam-ocr",                           "need_root": False},
    ],
    "PAYLD": [
        {"name": "C2 Listener",       "desc": "Encrypted C2 shell (socat)",      "cmd": "quick-c2 listen",                   "need_root": False, "args": ["PORT"]},
        {"name": "C2 Payloads",       "desc": "Generate shell one-liners",       "cmd": "quick-c2 payload",                  "need_root": False},
        {"name": "Stabilize Shell",   "desc": "PTY/TTY upgrade cheatsheet",      "cmd": "revshell-stabilize",                "need_root": False},
        {"name": "Crack Hashes",      "desc": "John the Ripper password cracker", "cmd": "john",                              "need_root": False},
        {"name": "Brute Force Creds", "desc": "Hydra online credential cracker", "cmd": "hydra",                             "need_root": False},
        {"name": "USB Ducky Mode",    "desc": "Switch USB-C to HID keyboard",    "cmd": "sudo usb-gadget-mode hid",          "need_root": True},
        {"name": "USB Mass Storage",  "desc": "Switch USB-C to flash drive",     "cmd": "sudo usb-gadget-mode mass",         "need_root": True},
        {"name": "USB Network",       "desc": "Switch USB-C to network adapter", "cmd": "sudo usb-gadget-mode ncm",          "need_root": True},
    ],
    "RADIO": [
        {"name": "Wi-Fi Walkie Talkie", "desc": "Local Push-To-Talk Radio",      "cmd": "WALKIE_TALKIE",                     "need_root": False},
        {"name": "SDR Scan",          "desc": "Frequency sweep via RTL-SDR",     "cmd": "sudo sdr-scan",                     "need_root": True,  "args": ["FREQ_RANGE"]},
        {"name": "RF Capture",        "desc": "Raw IQ signal capture",           "cmd": "sudo rf-capture",                   "need_root": True,  "args": ["FREQ"]},
        {"name": "GPIO Probe",        "desc": "Enumerate I2C/SPI/UART devices",  "cmd": "sudo gpio-probe",                   "need_root": True},
    ],
    "MEDIA": [
        {"name": "Danish WebRadio",   "desc": "Stream Danish Radio Channels",    "cmd": "MEDIA_PLAYER:RADIO",                "need_root": False},
        {"name": "Local Music",       "desc": "Play MP3s from /opt/cardputer",   "cmd": "MEDIA_PLAYER:MUSIC",                "need_root": False},
        {"name": "Stop Playback",     "desc": "Kill audio background process",   "cmd": "killall ffplay",                    "need_root": False},
    ],
    "SHELL": [
        {"name": "Quick Terminal",    "desc": "Open bash shell",                 "cmd": "bash",                              "need_root": False},
        {"name": "Root Terminal",     "desc": "Open root shell",                 "cmd": "sudo bash",                         "need_root": True},
        {"name": "OpenCode",          "desc": "AI-assisted code editor",         "cmd": "opencode-session",                  "need_root": False},
        {"name": "WiFi Setup",        "desc": "Configure WiFi connection",       "cmd": "sudo cardputer-wifi-setup",         "need_root": True},
        {"name": "WiFi Toggle",       "desc": "Toggle wlan0 on/off",             "cmd": "sudo cardputer-wifi-toggle",        "need_root": True},
    ],
    "SYS": [
        {"name": "Battery Status",    "desc": "Show battery level + voltage",    "cmd": "cardputer-battery",                 "need_root": False},
        {"name": "MonsterC5 Status",  "desc": "Check MonsterC5 connection",      "cmd": "monsterctl status",                 "need_root": False},
        {"name": "Dongle Status",     "desc": "RTL8821CU dongle manager",        "cmd": "dongle-setup status",               "need_root": False},
        {"name": "MAC Rotate",        "desc": "Randomize MAC address",           "cmd": "sudo mac-rotate wlan0",             "need_root": True},
        {"name": "Performance Mode",  "desc": "1GHz quad, all radios on",        "cmd": "sudo power-mode performance",       "need_root": True},
        {"name": "Balanced Mode",     "desc": "800MHz dual, WiFi only",          "cmd": "sudo power-mode balanced",          "need_root": True},
        {"name": "Stealth Mode",      "desc": "600MHz single, radios off",       "cmd": "sudo power-mode stealth",           "need_root": True},
        {"name": "Loot Organize",     "desc": "Sort and compress captures",      "cmd": "loot-organize",                     "need_root": False},
        {"name": "DoH Proxy",         "desc": "DNS-over-HTTPS tunnel",            "cmd": "doh-proxy start",                    "need_root": True},
        {"name": "PANIC",             "desc": "Kill all + wipe + sanitize",      "cmd": "panic",                             "need_root": False},
        {"name": "System Info",       "desc": "Show OS + hardware info",         "cmd": "cat /etc/zeroday-release; uname -a; free -m; df -h /", "need_root": False},
    ],
    "OPENCODE": [
        {"name": "Open Code Editor",  "desc": "Launch OpenCode IDE",             "cmd": "opencode-session",                  "need_root": False},
        {"name": "Open Workspace",    "desc": "OpenCode in /opt/cardputer",      "cmd": "opencode-session /opt/cardputer",   "need_root": False},
        {"name": "Open Loot Dir",     "desc": "OpenCode in loot directory",      "cmd": "opencode-session /opt/cardputer/loot", "need_root": False},
        {"name": "Open Config Dir",   "desc": "OpenCode in config directory",    "cmd": "opencode-session /opt/cardputer/config", "need_root": False},
    ],
}

# ==========================================
# PROCEDURAL ICON RENDERING
# ==========================================
def draw_icon(surface: pygame.Surface, category: str, x: int, y: int, size: int = 24, color: tuple = (255,255,255)) -> None:
    cx, cy = x + size // 2, y + size // 2
    
    if category == "WIFI":
        pygame.draw.arc(surface, color, (x+2, y+2, 20, 20), math.pi/4, 3*math.pi/4, 2)
        pygame.draw.arc(surface, color, (x+5, y+5, 14, 14), math.pi/4, 3*math.pi/4, 2)
        pygame.draw.arc(surface, color, (x+8, y+8, 8, 8), math.pi/4, 3*math.pi/4, 2)
        pygame.draw.circle(surface, color, (cx, cy + 6), 2)
        
    elif category == "M5MONSTER":
        pygame.draw.rect(surface, color, (x+6, y+6, 4, 3))
        pygame.draw.rect(surface, color, (x+14, y+6, 4, 3))
        pts = [(x+4, y+14), (x+8, y+18), (x+12, y+14), (x+16, y+18), (x+20, y+14)]
        pygame.draw.lines(surface, color, False, pts, 2)
        
    elif category == "NET":
        pts = [(cx, y+4), (x+4, y+18), (x+20, y+18)]
        pygame.draw.lines(surface, color, True, pts, 2)
        pygame.draw.circle(surface, color, pts[0], 3)
        pygame.draw.circle(surface, color, pts[1], 3)
        pygame.draw.circle(surface, color, pts[2], 3)
        pygame.draw.circle(surface, color, (cx, cy+2), 4)
        pygame.draw.line(surface, color, pts[0], (cx, cy+2), 2)
        pygame.draw.line(surface, color, pts[1], (cx, cy+2), 2)
        pygame.draw.line(surface, color, pts[2], (cx, cy+2), 2)
        
    elif category == "BT":
        pygame.draw.line(surface, color, (cx, y+2), (cx, y+22), 2)
        pts1 = [(cx, y+2), (x+18, y+7), (cx, y+12)]
        pts2 = [(cx, y+12), (x+18, y+17), (cx, y+22)]
        pygame.draw.lines(surface, color, False, pts1, 2)
        pygame.draw.lines(surface, color, False, pts2, 2)
        pygame.draw.line(surface, color, (x+6, y+7), (cx, y+12), 2)
        pygame.draw.line(surface, color, (x+6, y+17), (cx, y+12), 2)
        
    elif category == "IR":
        pts = [(x+4, y+8), (x+8, y+4), (x+12, y+8)]
        pygame.draw.lines(surface, color, False, pts, 2)
        pts2 = [(x+4, y+16), (x+8, y+12), (x+12, y+16)]
        pygame.draw.lines(surface, color, False, pts2, 2)
        pygame.draw.polygon(surface, color, [(x+14, y+10), (x+22, y+10), (x+18, y+6)])
        pygame.draw.polygon(surface, color, [(x+14, y+14), (x+22, y+14), (x+18, y+18)])
        
    elif category == "CAM":
        pygame.draw.rect(surface, color, (x+2, y+6, 20, 14), 2)
        pygame.draw.circle(surface, color, (cx, cy+2), 4, 2)
        pygame.draw.rect(surface, color, (x+8, y+3, 8, 3))
        
    elif category == "PAYLD":
        pygame.draw.rect(surface, color, (x+8, y+8, 8, 12), 2)
        pygame.draw.line(surface, color, (x+12, y+4), (x+12, y+8), 2)
        pygame.draw.line(surface, color, (x+9, y+4), (x+15, y+4), 2)
        pygame.draw.polygon(surface, color, [(x+8, y+20), (x+16, y+20), (x+12, y+24)])
        
    elif category == "RADIO":
        pygame.draw.line(surface, color, (cx, y+10), (cx, y+22), 2)
        pygame.draw.arc(surface, color, (cx-6, y+2, 12, 12), 0, math.pi, 2)
        pygame.draw.arc(surface, color, (cx-4, y+5, 8, 8), 0, math.pi, 2)
        pygame.draw.circle(surface, color, (cx, y+10), 2)
        
    elif category == "MEDIA":
        # Musical note icon — simplified for 24px
        pygame.draw.line(surface, color, (x+8, y+4), (x+8, y+18), 2)
        pygame.draw.circle(surface, color, (x+6, y+18), 3, 2)
        
    elif category == "SHELL":
        pygame.draw.rect(surface, color, (x+2, y+4, 20, 16), 2)
        pygame.draw.lines(surface, color, False, [(x+5, y+8), (x+9, y+12), (x+5, y+16)], 2)
        pygame.draw.line(surface, color, (x+11, y+16), (x+17, y+16), 2)
        
    elif category == "SYS":
        pygame.draw.circle(surface, color, (cx, cy), 6, 2)
        for angle in range(0, 360, 45):
            rad = math.radians(angle)
            r1, r2 = 6, 10
            x1 = cx + math.cos(rad) * r1
            y1 = cy + math.sin(rad) * r1
            x2 = cx + math.cos(rad) * r2
            y2 = cy + math.sin(rad) * r2
            pygame.draw.line(surface, color, (x1, y1), (x2, y2), 3)
            
    elif category == "OPEN":
        pygame.draw.lines(surface, color, False, [(x+6, y+6), (x+2, y+12), (x+6, y+18)], 2)
        pygame.draw.lines(surface, color, False, [(x+18, y+6), (x+22, y+12), (x+18, y+18)], 2)
        pygame.draw.line(surface, color, (x+14, y+6), (x+10, y+18), 2)
        
    else:
        # Fallback empty rect
        pygame.draw.rect(surface, color, (x+4, y+4, 16, 16), 2)

# ==========================================
# APPLICATION CLASS
# ==========================================
class CyberLauncher:
    def __init__(self):
        pygame.init()
        self.screen = pygame.display.set_mode((SCREEN_W, SCREEN_H), pygame.NOFRAME)
        pygame.display.set_caption("ZERO-DAY OS")
        pygame.key.set_repeat(300, 50)
        
        # Load fonts (fallback to monospace if TTF missing)
        try:
            base_dir = os.path.dirname(os.path.abspath(__file__))
            font_path = os.path.join(base_dir, "assets", "fonts", "RobotoMono-Regular.ttf")
            self.font_title = pygame.font.Font(font_path, 12)
            self.font_title.set_bold(True)
            self.font_label = pygame.font.Font(font_path, 10)
            self.font_desc  = pygame.font.Font(font_path, 8)
            self.font_cmd   = pygame.font.Font(font_path, 9)
        except Exception:
            self.font_title = pygame.font.SysFont("monospace", 12, bold=True)
            self.font_label = pygame.font.SysFont("monospace", 10)
            self.font_desc  = pygame.font.SysFont("monospace", 8)
            self.font_cmd   = pygame.font.SysFont("monospace", 9)
            
        self.clock = pygame.time.Clock()
        
        # States: SPLASH, HOME, LIST, ACTION, PROMPT, WALKIE_TALKIE, MEDIA_PLAYER
        self.state = "SPLASH"
        self.splash_start = pygame.time.get_ticks()
        
        base_dir = os.path.dirname(os.path.abspath(__file__))
        try:
            self.logo_img = pygame.image.load(os.path.join(base_dir, "assets", "logo_splash.png")).convert_alpha()
        except:
            self.logo_img = None
            
        # Load Custom PNG Icons
        self.icons = {}
        icon_dir = os.path.join(base_dir, "assets", "icons")
        for cat in CATEGORIES:
            try:
                ipath = os.path.join(icon_dir, f"{cat['key']}.png")
                img = pygame.image.load(ipath).convert()
                img.set_colorkey((0, 0, 0))
                self.icons[cat["key"]] = img
            except:
                self.icons[cat["key"]] = None
                
        # Premium UX setup
        self.anim_offset = 0.0
        try:
            self.bg_img = pygame.image.load(os.path.join(base_dir, "assets", "bg.png")).convert()
        except:
            self.bg_img = None
            
        try:
            pygame.mixer.init(frequency=44100, size=-16, channels=1, buffer=512)
            self.tick_snd = pygame.mixer.Sound(os.path.join(base_dir, "assets", "audio", "tick.wav"))
        except:
            self.tick_snd = None
                
        self.running = True
        
        # Cursors / Navigation
        self.home_idx = 0
        self.list_idx = 0
        self.list_scroll = 0
        self.prompt_idx = 0
        
        self.current_cat = None
        self.current_tool = None
        
        self.args_values = {}
        self.render_cache = {}
        
        # Media Player state
        self.media_process = None
        self.media_mode = None
        self.media_stations = ["DR_P1", "DR_P3", "NOVA", "POPFM"]
        self.media_station_idx = 0
        self.media_fft = [0]*16
        
        # Walkie Talkie state (thread-safe via threading.Event)
        self.wt_socket = None
        self.wt_receiver_thread = None
        self.wt_aplay_proc = None
        self.wt_arecord_proc = None
        self.wt_receiving = False
        self.wt_transmit_event = threading.Event()
        self.wt_tx_start_time = 0

        # File Browser state
        self.fb_path = "/"
        self.fb_items = []
        self.fb_idx = 0
        self.fb_scroll = 0

    def load_directory(self, path):
        try:
            items = os.listdir(path)
        except PermissionError:
            return False
            
        self.fb_items = []
        if path != "/":
            self.fb_items.append({"name": "..", "is_dir": True})
            
        dirs, files = [], []
        for item in items:
            full = os.path.join(path, item)
            if os.path.isdir(full): dirs.append({"name": item, "is_dir": True})
            else: files.append({"name": item, "is_dir": False})
            
        dirs.sort(key=lambda x: x["name"].lower())
        files.sort(key=lambda x: x["name"].lower())
        self.fb_items.extend(dirs)
        self.fb_items.extend(files)
        
        self.fb_path = path
        self.fb_idx = 0
        self.fb_scroll = 0
        return True

    def play_tick(self):
        if self.tick_snd:
            self.tick_snd.play()

    def get_text_surface(self, text, font, color):
        key = f"{text}_{font}_{color}"
        if key not in self.render_cache:
            self.render_cache[key] = font.render(text, False, color)
        return self.render_cache[key]

    def draw_top_banner(self, title, color=TEXT_PRIMARY, right_text=""):
        pygame.draw.rect(self.screen, DIM_BG, (0, 0, SCREEN_W, 16))
        pygame.draw.line(self.screen, color, (0, 16), (SCREEN_W, 16), 1)
        
        tsurf = self.get_text_surface(title, self.font_title, color)
        self.screen.blit(tsurf, (4, 2))
        
        if right_text:
            rsurf = self.get_text_surface(right_text, self.font_label, TEXT_SECONDARY)
            self.screen.blit(rsurf, (SCREEN_W - rsurf.get_width() - 4, 3))

    def render_splash(self):
        self.screen.fill(BG_COLOR)
        if hasattr(self, "logo_img") and self.logo_img:
            lx = (SCREEN_W - self.logo_img.get_width()) // 2
            ly = (SCREEN_H - self.logo_img.get_height()) // 2
            self.screen.blit(self.logo_img, (lx, ly))
            
        vsurf = self.get_text_surface("ZERO-DAY OS v1.0.0", self.font_label, TEXT_SECONDARY)
        self.screen.blit(vsurf, (SCREEN_W - vsurf.get_width() - 5, SCREEN_H - 15))
        
        if pygame.time.get_ticks() - self.splash_start > 2500:
            self.state = "HOME"

    def render_home(self):
        if self.bg_img:
            self.screen.blit(self.bg_img, (0, 0))
        else:
            self.screen.fill(BG_COLOR)
            
        # Lerp animation
        self.anim_offset += (0.0 - self.anim_offset) * 0.3
        if abs(self.anim_offset) < 0.01:
            self.anim_offset = 0.0
            
        slide_x = int(self.anim_offset * (SCREEN_W // 2))
        
        cat = CATEGORIES[self.home_idx]
        color = cat["color"]
        
        self.draw_top_banner("ZERO-DAY OS", right_text="v1.0")
        
        cx = SCREEN_W // 2
        cy = SCREEN_H // 2
        
        prev_idx = (self.home_idx - 1) % len(CATEGORIES)
        prev_cat = CATEGORIES[prev_idx]
        prev_img = self.icons.get(prev_cat["key"])
        if prev_img:
            small_prev = pygame.transform.scale(prev_img, (48, 48))
            small_prev.set_alpha(80)
            self.screen.blit(small_prev, (10 + slide_x, cy - 24), special_flags=pygame.BLEND_RGBA_ADD)
            
        next_idx = (self.home_idx + 1) % len(CATEGORIES)
        next_cat = CATEGORIES[next_idx]
        next_img = self.icons.get(next_cat["key"])
        if next_img:
            small_next = pygame.transform.scale(next_img, (48, 48))
            small_next.set_alpha(80)
            self.screen.blit(small_next, (SCREEN_W - 58 + slide_x, cy - 24), special_flags=pygame.BLEND_RGBA_ADD)
            
        center_img = self.icons.get(cat["key"])
        if center_img:
            pulse = abs(math.sin(pygame.time.get_ticks() / 400.0))
            center_glow = center_img.copy()
            center_glow.set_alpha(int(150 + 105 * pulse))
            self.screen.blit(center_glow, (cx - 48 + slide_x, cy - 48), special_flags=pygame.BLEND_RGBA_ADD)
            
        tsurf = self.get_text_surface(f"<{cat['name']}>", self.font_title, color)
        self.screen.blit(tsurf, (cx - tsurf.get_width()//2 + slide_x, cy + 55))
        
        hint = self.get_text_surface("L/R: Navigate  Enter: Select", self.font_label, TEXT_SECONDARY)
        self.screen.blit(hint, (cx - hint.get_width()//2, SCREEN_H - 15))

    def render_file_browser(self):
        if self.bg_img: self.screen.blit(self.bg_img, (0, 0))
        else: self.screen.fill(BG_COLOR)
        
        self.draw_top_banner("FILE BROWSER", color=CATEGORIES[11]["color"], right_text="← Back(Esc)")
        
        path_surf = self.get_text_surface(f"Path: {self.fb_path}", self.font_cmd, TEXT_SECONDARY)
        self.screen.blit(path_surf, (10, 16))
        pygame.draw.line(self.screen, DIM_BG, (10, 26), (SCREEN_W-10, 26))
        
        if not self.fb_items:
            emp = self.get_text_surface("[Empty or Permission Denied]", self.font_label, TEXT_SECONDARY)
            self.screen.blit(emp, (20, 40))
            return
            
        y = 30
        for i in range(self.fb_scroll, min(len(self.fb_items), self.fb_scroll + 9)):
            item = self.fb_items[i]
            is_selected = (i == self.fb_idx)
            
            if is_selected:
                pygame.draw.rect(self.screen, DIM_BG, (8, y-2, SCREEN_W-16, 14))
                
            icon = "[D]" if item["is_dir"] else "[F]"
            color = CATEGORIES[11]["color"] if item["is_dir"] else TEXT_PRIMARY
            if is_selected: color = (255, 255, 255)
            
            t = self.get_text_surface(f"{icon} {item['name']}", self.font_label, color)
            self.screen.blit(t, (12, y))
            y += 15
            
        if len(self.fb_items) > 9:
            sb_h = max(10, int((9 / len(self.fb_items)) * 135))
            sb_y = 30 + int((self.fb_scroll / max(1, len(self.fb_items) - 9)) * (135 - sb_h))
            pygame.draw.rect(self.screen, DIM_BG, (SCREEN_W - 6, 30, 4, 135))
            pygame.draw.rect(self.screen, TEXT_SECONDARY, (SCREEN_W - 6, sb_y, 4, sb_h))

    def render_list(self):
        self.screen.fill(BG_COLOR)
        cat = CATEGORIES[self.home_idx]
        self.draw_top_banner(cat["name"], cat["color"], "← Back(Esc)")
        
        draw_icon(self.screen, cat["key"], SCREEN_W - 90, 0, 16, cat["color"])
        
        tools = TOOLS.get(cat["key"], [])
        item_h = 28
        visible_items = 5
        
        # Adjust scroll
        if self.list_idx < self.list_scroll:
            self.list_scroll = self.list_idx
        elif self.list_idx >= self.list_scroll + visible_items:
            self.list_scroll = self.list_idx - visible_items + 1
            
        y_offset = 18
        for i in range(self.list_scroll, min(len(tools), self.list_scroll + visible_items)):
            tool = tools[i]
            is_focused = (i == self.list_idx)
            
            y = y_offset + (i - self.list_scroll) * item_h
            
            if is_focused:
                c = cat["color"]
                hl_color = (c[0]//3, c[1]//3, c[2]//3)
                pygame.draw.rect(self.screen, hl_color, (0, y, SCREEN_W, item_h))
                pygame.draw.line(self.screen, cat["color"], (0, y), (0, y+item_h), 2)
            
            color = TEXT_WHITE if is_focused else TEXT_SECONDARY
            tsurf = self.get_text_surface(tool["name"], self.font_label, color)
            self.screen.blit(tsurf, (10, y + 3))
            
            dsurf = self.get_text_surface(tool["desc"], self.font_desc, TEXT_SECONDARY)
            self.screen.blit(dsurf, (10, y + 16))
            
        # Scrollbar
        if len(tools) > visible_items:
            bar_h = int((visible_items / len(tools)) * (SCREEN_H - 18))
            bar_y = 18 + int((self.list_scroll / len(tools)) * (SCREEN_H - 18))
            pygame.draw.rect(self.screen, DIM_BG, (SCREEN_W - 4, 18, 4, SCREEN_H - 18))
            pygame.draw.rect(self.screen, cat["color"], (SCREEN_W - 4, bar_y, 4, bar_h))

    def render_action(self):
        self.screen.fill(BG_COLOR)
        cat = CATEGORIES[self.home_idx]
        self.draw_top_banner("EXECUTE", cat["color"])
        
        tool = TOOLS[cat["key"]][self.list_idx]
        
        tsurf = self.get_text_surface(tool["name"], self.font_title, TEXT_WHITE)
        self.screen.blit(tsurf, (10, 30))
        
        dsurf = self.get_text_surface(tool["desc"], self.font_label, TEXT_SECONDARY)
        self.screen.blit(dsurf, (10, 48))
        
        # Command syntax highlighting
        cmd = tool["cmd"]
        if tool.get("need_root"):
            if not cmd.startswith("sudo"): cmd = "sudo " + cmd
            
        pygame.draw.rect(self.screen, DIM_BG, (10, 70, SCREEN_W-20, 40))
        pygame.draw.rect(self.screen, cat["color"], (10, 70, SCREEN_W-20, 40), 1)
        
        # Basic highlight rendering
        parts = cmd.split(" ")
        x_off = 15
        for p in parts:
            c = CMD_TEXT
            if p.startswith("-"): c = CMD_FLAG
            elif "/" in p: c = CMD_PATH
            elif p in ["sudo", "bash"]: c = cat["color"]
            
            psurf = self.get_text_surface(p + " ", self.font_cmd, c)
            self.screen.blit(psurf, (x_off, 85))
            x_off += psurf.get_width()
            
        bot_surf = self.get_text_surface("Enter:Run   Esc:Back", self.font_label, TEXT_SECONDARY)
        self.screen.blit(bot_surf, (10, SCREEN_H - 20))

    def render_prompt(self):
        self.screen.fill(BG_COLOR)
        cat = CATEGORIES[self.home_idx]
        tool = TOOLS[cat["key"]][self.list_idx]
        self.draw_top_banner(tool["name"], cat["color"])
        
        args = tool.get("args", [])
        
        y_offset = 25
        for i, arg in enumerate(args):
            is_focused = (i == self.prompt_idx)
            val = self.args_values.get(arg, "")
            
            lsurf = self.get_text_surface(arg + ":", self.font_label, cat["color"] if is_focused else TEXT_SECONDARY)
            self.screen.blit(lsurf, (10, y_offset))
            
            pygame.draw.rect(self.screen, TEXT_WHITE if is_focused else DIM_BG, (80, y_offset-2, 200, 16), 1 if is_focused else 0)
            
            vsurf = self.get_text_surface(val + ("_" if is_focused and (pygame.time.get_ticks()//500)%2==0 else ""), self.font_cmd, TEXT_WHITE if is_focused else TEXT_SECONDARY)
            self.screen.blit(vsurf, (85, y_offset))
            
            y_offset += 25
            
        # Command preview (sanitized)
        cmd = self.sanitize_cmd(tool, self.args_values)
            
        pygame.draw.rect(self.screen, DIM_BG, (10, SCREEN_H - 45, SCREEN_W-20, 20))
        csurf = self.get_text_surface(cmd, self.font_cmd, TEXT_SECONDARY)
        self.screen.blit(csurf, (15, SCREEN_H - 42))

        bot_surf = self.get_text_surface("Tab:Next   Enter:Run   Esc:Back", self.font_label, TEXT_SECONDARY)
        self.screen.blit(bot_surf, (10, SCREEN_H - 20))

    # ==========================================
    # WALKIE TALKIE
    # ==========================================
    def start_walkie_talkie(self):
        # Check audio tools availability
        for tool in ["arecord", "aplay"]:
            if not os.path.exists(f"/usr/bin/{tool}"):
                print(f"[!] Walkie Talkie requires {tool} (install alsa-utils)")
                self.state = "LIST"
                return
        
        # Create UDP socket
        self.wt_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        try:
            self.wt_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
        except (AttributeError, OSError):
            pass
        self.wt_socket.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        try:
            self.wt_socket.bind(("", WT_PORT))
        except OSError as e:
            print(f"[!] Walkie Talkie: Port {WT_PORT} in use: {e}")
            self.wt_socket.close()
            self.wt_socket = None
            self.state = "LIST"
            return
        self.wt_socket.setblocking(False)
        
        # Start playback process
        try:
            self.wt_aplay_proc = subprocess.Popen(
                ["aplay", "-q", "-f", "S16_LE", "-c", "1", "-r", "16000"],
                stdin=subprocess.PIPE,
                stderr=subprocess.DEVNULL
            )
        except FileNotFoundError:
            print("[!] aplay not found — install alsa-utils")
            self.wt_socket.close()
            self.wt_socket = None
            self.state = "LIST"
            return
        
        self.wt_receiving = False
        self.wt_transmit_event.clear()
        
        # Receiver thread: plays incoming audio when not transmitting
        def receiver_loop():
            while self.state == "WALKIE_TALKIE" and self.wt_socket:
                try:
                    ready = select.select([self.wt_socket], [], [], 0.1)
                    if ready[0]:
                        data, addr = self.wt_socket.recvfrom(4096)
                        # Ignore our own transmissions (check source port heuristic)
                        if data.startswith(b"WT1") and len(data) > 3:
                            self.wt_receiving = True
                            # Only play audio when NOT transmitting (prevent feedback)
                            if not self.wt_transmit_event.is_set():
                                if self.wt_aplay_proc and self.wt_aplay_proc.stdin:
                                    try:
                                        self.wt_aplay_proc.stdin.write(data[3:])
                                        self.wt_aplay_proc.stdin.flush()
                                    except Exception:
                                        pass
                        else:
                            self.wt_receiving = False
                    else:
                        self.wt_receiving = False
                except Exception:
                    pass
        
        self.wt_receiver_thread = threading.Thread(target=receiver_loop, daemon=True)
        self.wt_receiver_thread.start()

    def stop_walkie_talkie(self):
        """Clean up all walkie talkie resources."""
        self.wt_transmit_event.clear()
        # Stop recording first
        if self.wt_arecord_proc:
            try:
                self.wt_arecord_proc.kill()
                self.wt_arecord_proc.wait(timeout=2)
            except Exception:
                pass
            self.wt_arecord_proc = None
        # Stop playback
        if self.wt_aplay_proc:
            try:
                self.wt_aplay_proc.kill()
                self.wt_aplay_proc.wait(timeout=2)
            except Exception:
                pass
            self.wt_aplay_proc = None
        # Close socket
        if self.wt_socket:
            try:
                self.wt_socket.close()
            except Exception:
                pass
            self.wt_socket = None
        self.wt_receiving = False

    def render_walkie_talkie(self):
        self.screen.fill(BG_COLOR)
        cat = CATEGORIES[self.home_idx]
        self.draw_top_banner("WI-FI WALKIE TALKIE", cat["color"])
        
        # PTT timeout warning
        is_transmitting = self.wt_transmit_event.is_set()
        elapsed = pygame.time.get_ticks() - self.wt_tx_start_time if is_transmitting else 0
        timeout_warning = is_transmitting and elapsed > (WT_MAX_TX_SECS * 1000 - 5000)  # Warn 5s before cutoff
        
        status_color = CMD_FLAG if is_transmitting else (cat["color"] if self.wt_receiving else TEXT_SECONDARY)
        status_text = "TRANSMITTING" if is_transmitting else ("RECEIVING" if self.wt_receiving else "IDLE")
        
        # Blinking effect for TX
        if is_transmitting and (pygame.time.get_ticks()//250)%2 == 0:
            status_color = BG_COLOR
            
        # Timeout warning
        if timeout_warning:
            wsurf = self.get_text_surface(f"TIMEOUT IN {(WT_MAX_TX_SECS - elapsed//1000)}s", self.font_desc, CMD_FLAG)
            self.screen.blit(wsurf, ((SCREEN_W - wsurf.get_width())//2, 45))
        
        ssurf = self.get_text_surface(status_text, self.font_title, status_color)
        self.screen.blit(ssurf, ((SCREEN_W - ssurf.get_width())//2, 60))
        
        dsurf = self.get_text_surface("Hold [SPACE] to Talk", self.font_label, TEXT_WHITE)
        self.screen.blit(dsurf, ((SCREEN_W - dsurf.get_width())//2, 90))
        
        # Show port info
        psurf = self.get_text_surface(f"UDP:{WT_PORT} (broadcast)", self.font_desc, TEXT_SECONDARY)
        self.screen.blit(psurf, ((SCREEN_W - psurf.get_width())//2, 110))
        
        bot_surf = self.get_text_surface("Esc:Stop & Exit", self.font_label, TEXT_SECONDARY)
        self.screen.blit(bot_surf, (10, SCREEN_H - 20))

    # ==========================================
    # MEDIA PLAYER
    # ==========================================
    def launch_media(self):
        # Kill previous media process
        if self.media_process:
            try:
                self.media_process.kill()
                self.media_process.wait(timeout=3)
            except Exception:
                pass
            self.media_process = None
        # Kill any orphaned ffplay
        try:
            subprocess.run(["killall", "ffplay"], capture_output=True, timeout=3)
        except Exception:
            pass

        if self.media_mode == "RADIO":
            station = self.media_stations[self.media_station_idx]
            # Use direct ffplay with stream URL
            url = WT_RADIO_URLS.get(station, "")
            if not url:
                print(f"[!] No URL configured for station {station}")
                return
            try:
                self.media_process = subprocess.Popen(
                    ["ffplay", "-nodisp", "-autoexit", "-infbuf", "-loglevel", "quiet", url],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
            except FileNotFoundError:
                print("[!] ffplay not found — install with: apt install ffmpeg")
        elif self.media_mode == "MUSIC":
            music_dir = "/opt/cardputer/music"
            if not os.path.isdir(music_dir):
                os.makedirs(music_dir, exist_ok=True)
            # Build list of audio files (ffplay cannot play a directory)
            audio_files = sorted(
                glob.glob(os.path.join(music_dir, "*.mp3"))
                + glob.glob(os.path.join(music_dir, "*.flac"))
                + glob.glob(os.path.join(music_dir, "*.wav"))
                + glob.glob(os.path.join(music_dir, "*.ogg"))
                + glob.glob(os.path.join(music_dir, "*.aac"))
                + glob.glob(os.path.join(music_dir, "*.m4a"))
            )
            if not audio_files:
                print(f"[!] No audio files found in {music_dir}")
                return
            random.shuffle(audio_files)
            try:
                self.media_process = subprocess.Popen(
                    ["ffplay", "-nodisp", "-autoexit", "-loglevel", "quiet"]
                    + audio_files,
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
            except FileNotFoundError:
                print("[!] ffplay not found — install with: apt install ffmpeg")

    def render_media_player(self):
        self.screen.fill(BG_COLOR)
        cat = CATEGORIES[self.home_idx]
        title = "WEBRADIO" if self.media_mode == "RADIO" else "MUSIC PLAYER"
        self.draw_top_banner(title, cat["color"])
        
        # Now Playing info
        if self.media_mode == "RADIO":
            station = self.media_stations[self.media_station_idx]
            nsurf = self.get_text_surface(f"Station: {station}", self.font_title, TEXT_WHITE)
            dsurf = self.get_text_surface("< Left / Right > to change", self.font_label, TEXT_SECONDARY)
        else:
            nsurf = self.get_text_surface("Playing Local Music (Shuffle)", self.font_title, TEXT_WHITE)
            dsurf = self.get_text_surface("Dir: /opt/cardputer/music", self.font_label, TEXT_SECONDARY)
            
        self.screen.blit(nsurf, (10, 30))
        self.screen.blit(dsurf, (10, 45))
        
        # Procedural visualizer
        v_y = 120
        v_w = 12
        v_gap = 4
        start_x = (SCREEN_W - (16 * (v_w + v_gap))) // 2
        
        for i in range(16):
            # Smooth random FFT simulation
            target = random.randint(5, 40)
            if self.media_fft[i] < target: self.media_fft[i] += 4
            elif self.media_fft[i] > target: self.media_fft[i] -= 4
            h = max(5, self.media_fft[i])
            
            # Draw bar
            rect = (start_x + i * (v_w + v_gap), v_y - h, v_w, h)
            pygame.draw.rect(self.screen, cat["color"], rect)
            
        # Controls
        bot_surf = self.get_text_surface("Esc:Stop & Exit", self.font_label, TEXT_SECONDARY)
        self.screen.blit(bot_surf, (10, SCREEN_H - 20))

    # ==========================================
    # COMMAND EXECUTION
    # ==========================================
    def sanitize_cmd(self, tool, args_values):
        """Build command string with proper argument quoting and validation."""
        cmd = tool["cmd"]
        if tool.get("need_root") and os.geteuid() != 0:
            if not cmd.startswith("sudo"):
                cmd = f"sudo {cmd}"
        
        # Input validation for argument types
        validators = {
            "BSSID": r'^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$',
            "CHANNEL": r'^\d{1,2}$',
            "PORT": r'^\d{1,5}$',
            "TARGET": r'^[\w.\-/]+$',
            "MAC": r'^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$',
            "FREQ": r'^\d+(\.\d+)?$',
            "FREQ_RANGE": r'^\d+-\d+$',
            "SUBNET": r'^[\d./]+$',
            "DURATION": r'^\d+$',
            "PIVOT_HOST": r'^[\w.\-]+$',
            "CAP_FILE": r'^[\w.\-/]+$',
            "FILE": r'^[\w.\-/]+$',
            "AP_IFACE": r'^[\w]+$',
            "INET_IFACE": r'^[\w]+$',
            "ESSID": r'^[\x20-\x7e]{1,32}$',
            "PROBE_N": r'^\d+$',
            "TYPE": r'^[\w]+$',
            "IP": r'^[\d.]+$',
        }
        
        for arg in tool.get("args", []):
            val = args_values.get(arg, "")
            if not val:
                cmd += f" {shlex.quote(val)}"
                continue
            # Validate known argument types
            if arg in validators:
                if not re.match(validators[arg], val):
                    cmd += f" {shlex.quote(val)}"
                    continue
            cmd += f" {shlex.quote(val)}"
        
        if "extra" in tool:
            cmd += f" {tool['extra']}"
        
        return cmd

    def execute_command(self):
        cat_idx = self.home_idx
        cat = CATEGORIES[cat_idx]
        tool = TOOLS[cat["key"]][self.list_idx]
        
        cmd = tool["cmd"]
        
        # Walkie Talkie — inline mode, no terminal
        if cmd == "WALKIE_TALKIE":
            self.state = "WALKIE_TALKIE"
            self.start_walkie_talkie()
            return
            
        # Media Player — inline mode, no terminal
        if cmd.startswith("MEDIA_PLAYER:"):
            mode = cmd.split(":", 1)[1]
            if mode not in ("RADIO", "MUSIC"):
                return  # Invalid mode, don't launch
            self.media_mode = mode
            self.state = "MEDIA_PLAYER"
            self.launch_media()
            return
        
        cmd = self.sanitize_cmd(tool, self.args_values)
            
        pygame.quit()
        try:
            os.execvp("st", ["st", "-e", "bash", "-c", f"{cmd}; echo; echo '[Finished]'; read -p 'Press Enter...'"])
        except OSError:
            os.execvp("/bin/bash", ["/bin/bash", "-c", cmd])

    def handle_input(self):
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.running = False
                
            elif event.type == pygame.KEYDOWN:
                self.play_tick()
                if self.state == "SPLASH":
                    self.state = "HOME"
                elif self.state == "HOME":
                    if event.key == pygame.K_RIGHT:
                        self.home_idx = (self.home_idx + 1) % len(CATEGORIES)
                        self.anim_offset = 1.0
                    elif event.key == pygame.K_LEFT:
                        self.home_idx = (self.home_idx - 1) % len(CATEGORIES)
                        self.anim_offset = -1.0
                    elif event.key == pygame.K_RETURN:
                        cat = CATEGORIES[self.home_idx]
                        if cat["key"] == "OPEN":
                            self.state = "FILE_BROWSER"
                            self.load_directory("/")
                        else:
                            self.state = "LIST"
                            self.list_idx = 0
                            self.list_scroll = 0
                    elif event.key in (pygame.K_ESCAPE, pygame.K_q):
                        self.running = False
                        
                elif self.state == "FILE_BROWSER":
                    if event.key == pygame.K_UP:
                        self.fb_idx = max(0, self.fb_idx - 1)
                        if self.fb_idx < self.fb_scroll: self.fb_scroll = self.fb_idx
                    elif event.key == pygame.K_DOWN:
                        self.fb_idx = min(len(self.fb_items) - 1, self.fb_idx + 1)
                        if self.fb_idx >= self.fb_scroll + 9: self.fb_scroll = self.fb_idx - 8
                    elif event.key == pygame.K_RIGHT or event.key == pygame.K_RETURN:
                        if self.fb_items:
                            item = self.fb_items[self.fb_idx]
                            if item["is_dir"]:
                                new_path = os.path.abspath(os.path.join(self.fb_path, item["name"]))
                                self.load_directory(new_path)
                            else:
                                self.current_tool = {"name": f"View {item['name']}", "cmd": f"cat '{os.path.join(self.fb_path, item['name'])}' | less"}
                                self.state = "ACTION"
                    elif event.key == pygame.K_LEFT or event.key == pygame.K_ESCAPE:
                        if self.fb_path == "/":
                            self.state = "HOME"
                        else:
                            self.load_directory(os.path.dirname(self.fb_path))
                            
                elif self.state == "LIST":
                    cat_key = CATEGORIES[self.home_idx]["key"]
                    tools = TOOLS.get(cat_key, [])
                    if event.key == pygame.K_DOWN: self.list_idx = (self.list_idx + 1) % len(tools)
                    elif event.key == pygame.K_UP: self.list_idx = (self.list_idx - 1) % len(tools)
                    elif event.key == pygame.K_ESCAPE: self.state = "HOME"
                    elif event.key == pygame.K_RETURN:
                        tool = tools[self.list_idx]
                        if tool.get("args"):
                            self.state = "PROMPT"
                            self.prompt_idx = 0
                            self.args_values = {a: "" for a in tool["args"]}
                        else:
                            self.state = "ACTION"
                            
                elif self.state == "ACTION":
                    if event.key == pygame.K_ESCAPE: self.state = "LIST"
                    elif event.key == pygame.K_RETURN: self.execute_command()
                    
                elif self.state == "PROMPT":
                    cat_key = CATEGORIES[self.home_idx]["key"]
                    tool = TOOLS[cat_key][self.list_idx]
                    args = tool.get("args", [])
                    cur_arg = args[self.prompt_idx]
                    
                    if event.key == pygame.K_ESCAPE: self.state = "LIST"
                    elif event.key == pygame.K_TAB: self.prompt_idx = (self.prompt_idx + 1) % len(args)
                    elif event.key == pygame.K_RETURN: self.execute_command()
                    elif event.key == pygame.K_BACKSPACE:
                        self.args_values[cur_arg] = self.args_values[cur_arg][:-1]
                    else:
                        if event.unicode.isprintable():
                            self.args_values[cur_arg] += event.unicode

                elif self.state == "MEDIA_PLAYER":
                    if event.key == pygame.K_ESCAPE:
                        if self.media_process:
                            try:
                                self.media_process.kill()
                                self.media_process.wait(timeout=3)
                            except Exception:
                                pass
                            self.media_process = None
                        subprocess.run(["killall", "ffplay"], capture_output=True, timeout=3)
                        self.media_fft = [0]*16
                        self.state = "LIST"
                    elif self.media_mode == "RADIO":
                        if event.key == pygame.K_RIGHT:
                            self.media_station_idx = (self.media_station_idx + 1) % len(self.media_stations)
                            self.launch_media()
                        elif event.key == pygame.K_LEFT:
                            self.media_station_idx = (self.media_station_idx - 1) % len(self.media_stations)
                            self.launch_media()

                elif self.state == "WALKIE_TALKIE":
                    if event.key == pygame.K_ESCAPE:
                        self.stop_walkie_talkie()
                        self.state = "LIST"
                    elif event.key == pygame.K_SPACE and not self.wt_transmit_event.is_set():
                        self.wt_transmit_event.set()
                        self.wt_tx_start_time = pygame.time.get_ticks()
                        # Pause playback during transmit to prevent feedback
                        if self.wt_aplay_proc:
                            try:
                                self.wt_aplay_proc.send_signal(signal.SIGSTOP)
                            except Exception:
                                pass
                        try:
                            self.wt_arecord_proc = subprocess.Popen(
                                ["arecord", "-q", "-f", "S16_LE", "-c", "1", "-r", "16000"],
                                stdout=subprocess.PIPE,
                                stderr=subprocess.DEVNULL
                            )
                        except FileNotFoundError:
                            print("[!] arecord not found — install alsa-utils")
                            self.wt_transmit_event.clear()
                            return
                        
                        def transmit_loop():
                            while self.wt_transmit_event.is_set() and self.wt_arecord_proc:
                                # Timeout after WT_MAX_TX_SECS
                                if pygame.time.get_ticks() - self.wt_tx_start_time > WT_MAX_TX_SECS * 1000:
                                    self.wt_transmit_event.clear()
                                    break
                                try:
                                    chunk = self.wt_arecord_proc.stdout.read(1024)
                                    if not chunk:
                                        break
                                    if self.wt_socket:
                                        self.wt_socket.sendto(b"WT1" + chunk, ("255.255.255.255", WT_PORT))
                                except Exception:
                                    break
                        
                        threading.Thread(target=transmit_loop, daemon=True).start()

            elif event.type == pygame.KEYUP:
                if self.state == "WALKIE_TALKIE" and event.key == pygame.K_SPACE:
                    self.wt_transmit_event.clear()
                    if self.wt_arecord_proc:
                        try:
                            self.wt_arecord_proc.kill()
                            self.wt_arecord_proc.wait(timeout=2)
                        except Exception:
                            pass
                        self.wt_arecord_proc = None
                    # Resume playback after transmit
                    if self.wt_aplay_proc:
                        try:
                            self.wt_aplay_proc.send_signal(signal.SIGCONT)
                        except Exception:
                            pass

    def run(self):
        while self.running:
            self.handle_input()
            
            if self.state == "SPLASH": self.render_splash()
            elif self.state == "HOME": self.render_home()
            elif self.state == "LIST": self.render_list()
            elif self.state == "ACTION": self.render_action()
            elif self.state == "PROMPT": self.render_prompt()
            elif self.state == "MEDIA_PLAYER":
                self.render_media_player()
            elif self.state == "FILE_BROWSER":
                self.render_file_browser()
            elif self.state == "WALKIE_TALKIE": self.render_walkie_talkie()
            
            pygame.display.flip()
            self.clock.tick(FPS_TARGET)
        
        # Clean up all resources on exit
        self.stop_walkie_talkie()
        if self.media_process:
            try:
                self.media_process.kill()
            except Exception:
                pass
        pygame.quit()

if __name__ == "__main__":
    app = CyberLauncher()
    app.run()