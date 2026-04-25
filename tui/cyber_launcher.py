#!/usr/bin/env python3
"""ZERO-DAY OS — Cyber Launcher TUI
Flipper-style drill-down interface for the Cardputer Zero
Built with Textual for the 1.9" ST7789v3 display (320x170)"""

import os
import subprocess
import sys
from pathlib import Path
from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal, Vertical, VerticalScroll
from textual.screen import Screen
from textual.widgets import Button, Footer, Header, Label, Static

LOOT_DIR = Path("/opt/cardputer/loot")
WORKSPACE_DIR = Path("/opt/cardputer/workspace")

CATEGORIES = [
    {"name": "WIFI",     "icon": "WIFI",    "color": "#00ff41"},
    {"name": "M5MONSTER","icon": "MON",      "color": "#ff3300"},
    {"name": "NET",      "icon": "NET",     "color": "#00bfff"},
    {"name": "BT",       "icon": "BT",      "color": "#4169e1"},
    {"name": "IR",       "icon": "IR",      "color": "#ff6600"},
    {"name": "CAM",      "icon": "CAM",     "color": "#ff1493"},
    {"name": "PAYLD",    "icon": "PAYLD",   "color": "#ffd700"},
    {"name": "RADIO",    "icon": "RADIO",   "color": "#9400d3"},
    {"name": "SHELL",    "icon": "SHELL",    "color": "#ff4444"},
    {"name": "SYS",      "icon": "SYS",     "color": "#888888"},
    {"name": "OPEN",     "icon": "OPEN",    "color": "#00ffff"},
]

TOOLS = {
    "WIFI": [
        {"name": "Scan Networks",     "desc": "Quick WiFi survey (wlan0)",     "cmd": "sudo wifi-scan",                    "need_root": True},
        {"name": "Capture Handshake", "desc": "WPA handshake (use dongle!)",  "cmd": "sudo wifi-handshake wlan1",          "need_root": True, "args": ["BSSID", "CHANNEL"]},
        {"name": "PMKID Capture",     "desc": "PMKID capture (use dongle!)",  "cmd": "sudo wifi-pmkid wlan1",              "need_root": True, "args": ["BSSID", "CHANNEL"]},
        {"name": "Deauth Attack",     "desc": "Deauth target AP (use dongle!)", "cmd": "sudo wifi-deauth wlan1",           "need_root": True, "args": ["BSSID", "CHANNEL"]},
        {"name": "Evil Twin",         "desc": "Rogue AP + NAT",              "cmd": "sudo wifi-evil-twin",               "need_root": True, "args": ["AP_IFACE", "INET_IFACE", "ESSID"]},
        {"name": "Crack Handshake",  "desc": "Crack .cap with hashcat",     "cmd": "wifi-crack",                         "need_root": False, "args": ["CAP_FILE"]},
        {"name": "Monitor Toggle",    "desc": "Switch managed/monitor",      "cmd": "sudo wifi-monitor-toggle",           "need_root": True},
        {"name": "Dongle Monitor",    "desc": "Put dongle into monitor mode",  "cmd": "sudo dongle-setup monitor",        "need_root": True},
        {"name": "Dongle Managed",    "desc": "Put dongle into managed mode",  "cmd": "sudo dongle-setup managed",        "need_root": True},
    ],
    "M5MONSTER": [
        {"name": "JanOS TUI",         "desc": "Full interactive controller (recommended)", "cmd": "install-janos run",              "need_root": False},
        {"name": "Detect Board",      "desc": "Ping & detect MonsterC5 board",    "cmd": "monsterctl status",                 "need_root": False},
        {"name": "Scan Networks",      "desc": "WiFi scan via MonsterC5 ESP32C5", "cmd": "sudo monsterctl scan",             "need_root": True},
        {"name": "Deauth Attack",      "desc": "Deauthenticate selected APs",    "cmd": "sudo monsterctl deauth",           "need_root": True},
        {"name": "Evil Twin",         "desc": "Rogue AP + captive portal",      "cmd": "sudo monsterctl evil_twin",        "need_root": True},
        {"name": "WPA3 SAE Overflow", "desc": "SAE commit flood DoS",          "cmd": "sudo monsterctl sae_overflow",      "need_root": True},
        {"name": "Handshake Capture", "desc": "Capture 4-way handshake",       "cmd": "sudo monsterctl handshake",        "need_root": True},
        {"name": "Karma Attack",      "desc": "Respond to probe requests",     "cmd": "sudo monsterctl karma",            "need_root": True, "args": ["PROBE_N"]},
        {"name": "Wardrive",          "desc": "GPS-tagged WiFi scan",           "cmd": "sudo monsterctl wardrive",         "need_root": True},
        {"name": "Sniffer",           "desc": "WiFi packet sniffer",            "cmd": "sudo monsterctl sniffer",          "need_root": True},
        {"name": "Stop Attack",       "desc": "Stop current MonsterC5 attack",   "cmd": "monsterctl stop",                  "need_root": False},
        {"name": "Install JanOS-app", "desc": "Install interactive TUI controller", "cmd": "install-janos install",          "need_root": True},
    ],
    "NET": [
        {"name": "Discover Hosts",    "desc": "ARP scan + ping sweep",        "cmd": "sudo net-discover eth0",            "need_root": True, "args": ["SUBNET"]},
        {"name": "Quick Scan",        "desc": "Nmap top 100 ports",           "cmd": "net-quickscan",                      "need_root": False, "args": ["TARGET"]},
        {"name": "Web Scan",          "desc": "Nmap web ports + scripts",     "cmd": "net-quickscan",                      "need_root": False, "args": ["TARGET"], "extra": "web"},
        {"name": "Vuln Scan",         "desc": "Nmap vuln + nikto + whatweb",  "cmd": "sudo net-vulnscan",                  "need_root": True, "args": ["TARGET"]},
        {"name": "Full Scan",         "desc": "All 65535 ports + scripts",     "cmd": "net-quickscan",                      "need_root": False, "args": ["TARGET"], "extra": "full"},
        {"name": "Pivot (SOCKS)",     "desc": "SOCKS proxy via SSH",           "cmd": "net-pivot socks",                   "need_root": False, "args": ["PIVOT_HOST"]},
        {"name": "Pivot (Chisel)",    "desc": "TCP tunnel via chisel",        "cmd": "net-pivot chisel",                   "need_root": False},
    ],
    "BT": [
        {"name": "Scan Devices",      "desc": "BLE + Classic discovery",       "cmd": "sudo bt-scan",                      "need_root": True},
        {"name": "Deep Enumerate",    "desc": "Name, class, SDP, LMP",        "cmd": "sudo bt-deep",                      "need_root": True, "args": ["MAC"]},
        {"name": "BlueBorne Test",    "desc": "Test BlueBorne vuln",           "cmd": "sudo bt-attack blueborne",           "need_root": True, "args": ["MAC"]},
        {"name": "L2Ping Flood",      "desc": "L2CAP ping flood (DoS)",       "cmd": "sudo bt-attack l2ping_flood",       "need_root": True, "args": ["MAC"]},
        {"name": "RFCOMM Scan",       "desc": "Scan RFCOMM channels",          "cmd": "sudo bt-attack rfcomm_scan",        "need_root": True, "args": ["MAC"]},
        {"name": "GATT Enumerate",    "desc": "BLE services + handles",        "cmd": "sudo ble-gatt",                     "need_root": True, "args": ["MAC"]},
    ],
    "IR": [
        {"name": "Scan IR Signal",    "desc": "Capture remote signals",        "cmd": "sudo ir-scan",                      "need_root": True},
        {"name": "Replay Signal",      "desc": "Replay captured signal",       "cmd": "sudo ir-replay",                    "need_root": True, "args": ["FILE"]},
        {"name": "Brute Force TV",    "desc": "TV power codes (NEC)",          "cmd": "sudo ir-brute nec tv",              "need_root": True},
        {"name": "Brute Force AC",     "desc": "AC power codes",               "cmd": "sudo ir-brute nec ac",              "need_root": True},
    ],
    "CAM": [
        {"name": "Snap Photo",        "desc": "Capture still image",            "cmd": "cam-snap",                           "need_root": False},
        {"name": "Record Video",      "desc": "Record video clip",             "cmd": "cam-stream",                         "need_root": False, "args": ["DURATION"]},
        {"name": "OCR Capture",       "desc": "Photo + text recognition",      "cmd": "cam-ocr",                             "need_root": False},
    ],
    "PAYLD": [
        {"name": "Reverse Shell Gen", "desc": "Generate shell one-liners",    "cmd": "revshell-gen",                       "need_root": False, "args": ["TYPE", "IP"]},
        {"name": "Shell Listener",    "desc": "Start netcat listener",          "cmd": "revshell-listen",                    "need_root": False, "args": ["PORT"]},
        {"name": "Stabilize Shell",   "desc": "PTY/TTY upgrade cheatsheet",    "cmd": "revshell-stabilize",                 "need_root": False},
        {"name": "Craft Payload",     "desc": "msfvenom wrapper",               "cmd": "payload-craft",                      "need_root": False, "args": ["TYPE", "IP"]},
        {"name": "USB Ducky Mode",    "desc": "Switch USB-C to HID keyboard",   "cmd": "sudo usb-gadget-mode hid",           "need_root": True},
        {"name": "USB Mass Storage",   "desc": "Switch USB-C to flash drive",   "cmd": "sudo usb-gadget-mode mass",          "need_root": True},
        {"name": "USB Network",        "desc": "Switch USB-C to network adapter","cmd": "sudo usb-gadget-mode ncm",           "need_root": True},
    ],
    "RADIO": [
        {"name": "SDR Scan",          "desc": "Frequency sweep via RTL-SDR",   "cmd": "sudo sdr-scan",                      "need_root": True, "args": ["FREQ_RANGE"]},
        {"name": "RF Capture",         "desc": "Raw IQ capture",                "cmd": "sudo rf-capture",                    "need_root": True, "args": ["FREQ"]},
        {"name": "GPIO Probe",         "desc": "Enumerate I2C/SPI/UART devices", "cmd": "sudo gpio-probe",                  "need_root": True},
    ],
    "SHELL": [
        {"name": "Quick Terminal",    "desc": "Open bash shell",               "cmd": "bash",                                "need_root": False},
        {"name": "Root Terminal",      "desc": "Open root shell",               "cmd": "sudo bash",                           "need_root": True},
        {"name": "OpenCode",           "desc": "AI-assisted code editor",       "cmd": "opencode-session",                   "need_root": False},
        {"name": "WiFi Setup",         "desc": "Configure WiFi connection",     "cmd": "sudo cardputer-wifi-setup",           "need_root": True},
        {"name": "WiFi Toggle",        "desc": "Toggle wlan0 on/off",           "cmd": "sudo cardputer-wifi-toggle",          "need_root": True},
    ],
    "SYS": [
        {"name": "Battery Status",    "desc": "Show battery level + voltage",   "cmd": "cardputer-battery",                  "need_root": False},
        {"name": "MonsterC5 Status",  "desc": "Check MonsterC5 board connection", "cmd": "monsterctl status",               "need_root": False},
        {"name": "Dongle Status",    "desc": "RTL8821CU dongle manager",         "cmd": "dongle-setup status",               "need_root": False},
        {"name": "Performance Mode",  "desc": "1GHz quad, all radios",          "cmd": "sudo power-mode performance",        "need_root": True},
        {"name": "Balanced Mode",     "desc": "800MHz dual, WiFi only",          "cmd": "sudo power-mode balanced",            "need_root": True},
        {"name": "Stealth Mode",      "desc": "600MHz single, no radios",       "cmd": "sudo power-mode stealth",             "need_root": True},
        {"name": "PANIC",             "desc": "Kill all + wipe + sanitize",      "cmd": "panic",                               "need_root": False},
        {"name": "System Info",        "desc": "Show OS and hardware info",       "cmd": "cat /etc/zeroday-release; uname -a; free -m; df -h /", "need_root": False},
    ],
    "OPEN": [
        {"name": "Open Code Editor",   "desc": "Launch OpenCode IDE",            "cmd": "opencode-session",                    "need_root": False},
        {"name": "Open Workspace",     "desc": "OpenCode in /opt/cardputer",     "cmd": "opencode-session /opt/cardputer",     "need_root": False},
        {"name": "Open Loot Dir",      "desc": "OpenCode in loot directory",    "cmd": "opencode-session /opt/cardputer/loot", "need_root": False},
        {"name": "Open Config Dir",    "desc": "OpenCode in config directory",   "cmd": "opencode-session /opt/cardputer/config", "need_root": False},
    ],
}


class GridButton(Button):
    def __init__(self, category: dict) -> None:
        super().__init__(label=category["icon"], classes=["category-btn"], id=f"cat-{category['name']}")
        self.category = category


class ToolButton(Button):
    def __init__(self, tool: dict) -> None:
        super().__init__(label=f"{tool['name']}", classes=["tool-btn"], id=f"tool-{tool['name']}")
        self.tool = tool


class GridScreen(Screen):
    CSS = """
    GridScreen {
        layout: grid;
        grid-size: 4 3;
        grid-gutter: 1;
        padding: 1;
    }
    .category-btn {
        width: 100%;
        height: 100%;
        background: $surface;
        color: $text;
        border: tall $primary;
        text-style: bold;
        content-align: center middle;
    }
    .category-btn:focus {
        background: $primary;
        color: $surface;
    }
    """

    def compose(self) -> ComposeResult:
        for cat in CATEGORIES:
            btn = GridButton(cat)
            btn.styles.color = cat["color"]
            yield btn

    def on_button_pressed(self, event: ButtonPressed) -> None:
        cat_name = event.button.id.replace("cat-", "")
        if cat_name in TOOLS:
            self.app.push_screen(ToolListScreen(cat_name))


class ToolListScreen(Screen):
    CSS = """
    ToolListScreen {
        padding: 0;
    }
    .tool-list-container {
        height: 100%;
        padding: 1;
    }
    .tool-btn {
        width: 100%;
        height: 3;
        background: $surface;
        color: $text;
        border-left: tall $primary;
        padding: 0 1;
        text-align: left;
    }
    .tool-btn:focus {
        background: $primary;
        color: $surface;
        border-left: tall $warning;
    }
    .tool-desc {
        color: $text-muted;
        text-style: italic;
        height: 1;
        padding: 0 2;
    }
    """

    def __init__(self, category: str) -> None:
        super().__init__()
        self.category = category

    def compose(self) -> ComposeResult:
        yield Label(f"[bold]{self.category}[/bold]", classes="category-title")
        with VerticalScroll(classes="tool-list-container"):
            for tool in TOOLS.get(self.category, []):
                yield ToolButton(tool)
                yield Label(tool["desc"], classes="tool-desc")

    def on_button_pressed(self, event: ButtonPressed) -> None:
        tool = event.button.tool
        if tool.get("args"):
            self.app.push_screen(PromptScreen(tool))
        else:
            self.app.push_screen(ActionScreen(tool))


class PromptScreen(Screen):
    CSS = """
    PromptScreen {
        padding: 1;
    }
    .prompt-label {
        color: $primary;
        text-style: bold;
        margin: 0 0 1 0;
    }
    .prompt-input {
        width: 100%;
        margin: 0 0 1 0;
    }
    .prompt-info {
        color: $text-muted;
    }
    """

    def __init__(self, tool: dict) -> None:
        super().__init__()
        self.tool = tool

    def compose(self) -> ComposeResult:
        yield Label(f"[bold]{self.tool['name']}[/bold] — Arguments Required", classes="prompt-label")
        for arg in self.tool.get("args", []):
            yield Label(f"$arg:", classes="prompt-info")
        yield Label("", id="cmd-preview", classes="prompt-label")

    def key_enter(self) -> None:
        cmd = self.tool["cmd"]
        if self.tool.get("need_root") and os.getuid() != 0:
            cmd = f"sudo {cmd}"
        self.app.exit()
        os.system(f"st -e bash -c '{cmd}; bash'")


class ActionScreen(Screen):
    CSS = """
    ActionScreen {
        padding: 1;
    }
    .action-title {
        color: $primary;
        text-style: bold;
        margin: 0 0 1 0;
    }
    .action-cmd {
        color: $warning;
        background: $surface;
        padding: 1;
        margin: 0 0 1 0;
    }
    .action-desc {
        color: $text-muted;
    }
    """

    def __init__(self, tool: dict) -> None:
        super().__init__()
        self.tool = tool

    def compose(self) -> ComposeResult:
        yield Label(f"[bold]Execute:[/bold] {self.tool['name']}", classes="action-title")
        cmd = self.tool["cmd"]
        if self.tool.get("need_root") and os.getuid() != 0:
            cmd = f"sudo {cmd}"
        yield Label(f"$ {cmd}", classes="action-cmd")
        yield Label(self.tool["desc"], classes="action-desc")
        yield Label("", classes="action-desc")
        yield Label("[Enter] Execute  [Esc] Back  [Tab] Edit", classes="action-desc")

    def key_enter(self) -> None:
        cmd = self.tool["cmd"]
        if self.tool.get("need_root") and os.getuid() != 0:
            cmd = f"sudo {cmd}"
        self.app.exit()
        os.system(f"st -e bash -c '{cmd}; bash'")

    def key_tab(self) -> None:
        cmd = self.tool["cmd"]
        if self.tool.get("need_root") and os.getuid() != 0:
            cmd = f"sudo {cmd}"
        self.app.exit()
        os.system(f"st -e bash -c 'history -s \"{cmd}\"; bash'")


class CyberLauncher(App):
    CSS = """
    Screen {
        background: #0a0a0a;
        color: #00ff41;
    }
    """

    TITLE = "ZERO-DAY OS"
    SUB_TITLE = "Cyber Launcher"

    BINDINGS = [
        Binding("escape", "back", "Back", show=True),
        Binding("q", "quit", "Quit", show=True),
    ]

    SCREENS = {
        "grid": GridScreen,
    }

    def on_mount(self) -> None:
        self.push_screen("grid")

    def action_back(self) -> None:
        if len(self.screen_stack) > 1:
            self.pop_screen()


if __name__ == "__main__":
    app = CyberLauncher()
    app.run()