#!/bin/bash -e
# stage3/02-xorg-i3/01-run.sh — Install Xorg + i3 window manager

# Configure Xorg for ST7789v3 LCD (framebuffer)
mkdir -p "${ROOTFS_DIR}/etc/X11"
cat > "${ROOTFS_DIR}/etc/X11/xorg.conf" << 'XORG'
Section "Device"
    Identifier "ST7789"
    Driver "fbdev"
    Option "fbdev" "/dev/fb0"
    Option "ShadowFB" "off"
EndSection

Section "Screen"
    Identifier "LCD"
    Device "ST7789"
    DefaultDepth 16
    SubSection "Display"
        Depth 16
        Modes "320x240"
    EndSubSection
EndSection

Section "ServerLayout"
    Identifier "Default"
    Screen "LCD"
EndSection
XORG

# Install i3 configuration
mkdir -p "${ROOTFS_DIR}/etc/i3"
if [ -f "${BASE_DIR}/../configs/i3/config" ]; then
    cp "${BASE_DIR}/../configs/i3/config" "${ROOTFS_DIR}/etc/i3/config"
else
    cat > "${ROOTFS_DIR}/etc/i3/config" << 'I3CONF'
# /etc/i3/config — ZERO-DAY OS

# ─── Core ───
set $mod Mod1                          # Fn maps to Alt
font pango:Terminus 8                  # Tiny font for 1.9" screen
default_border none                    # No borders
default_floating_border none           # No floating borders
hide_edge_borders both                 # No edge borders
focus_follows_mouse no                 # Focus by keyboard only

# ─── Startup ───
exec_always --no-startup-id xset s off # No screensaver
exec_always --no-startup-id xset -dpms # No DPMS (we handle our own backlight)

# Auto-start the TUI
exec_always --no-startup-id st -e cyber_launcher

# ─── System Keybindings (Fn + Key) ───
bindsym $mod+Tab    exec --no-startup-id cyber_launcher
bindsym $mod+p      exec --no-startup-id panic
bindsym $mod+space  exec --no-startup-id stealth-backlight-toggle
bindsym $mod+Return exec --no-startup-id st -e tmux
bindsym $mod+q      kill
bindsym $mod+o      exec --no-startup-id opencode-session

# ─── Quick-Launch Keybindings ───
bindsym $mod+n      exec --no-startup-id st -e "sudo net-quickscan"
bindsym $mod+b      exec --no-startup-id st -e "sudo bt-scan"
bindsym $mod+s      exec --no-startup-id st -e "revshell-listen"
bindsym $mod+w      exec --no-startup-id cardputer-wifi-toggle
bindsym $mod+c      exec --no-startup-id cam-snap
bindsym $mod+i      exec --no-startup-id st -e "sudo ir-scan"
bindsym $mod+d      exec --no-startup-id dongle-setup status
bindsym $mod+a      exec --no-startup-id opencode-ask

# ─── i3 Bar ───
bar {
    mode dock
    position bottom
    height 12
    font pango:Terminus 6
    status_command i3status
    colors {
        background #000000
        statusline #00ff00
        focused_workspace #1a1a2e #00ff41 #000000
        active_workspace #000000 #00ff41 #000000
    }
}
I3CONF
fi

# Install i3status configuration
mkdir -p "${ROOTFS_DIR}/etc/i3status"
cat > "${ROOTFS_DIR}/etc/i3status.conf" << 'I3STAT'
general {
    output_format = "i3bar"
    colors = true
    interval = 5
}

order += "wireless wlan0"
order += "wireless wlan1"
order += "ethernet eth0"
order += "battery 0"
order += "cpu_temperature 0"
order += "load"
order += "tztime"

wireless wlan0 {
    format_up = "W0:%ip"
    format_down = "W0:OFF"
}

wireless wlan1 {
    format_up = "DNG:%ip"
    format_down = "DNG:OFF"
}

ethernet eth0 {
    format_up = "E:%ip"
    format_down = "E:OFF"
}

battery 0 {
    format = "BAT:%percentage%% %status"
    path = "/sys/class/power_supply/bq27220/uevent"
    low_threshold = 15
}

cpu_temperature 0 {
    format = "T:%degrees°C"
    path = "/sys/class/thermal/thermal_zone0/temp"
}

load {
    format = "L:%1min"
}

tztime {
    format = "%H:%M"
}
I3STAT

# Create .xsession for auto-startx
cat > "${ROOTFS_DIR}/root/.xsession" << 'XSESS'
#!/bin/sh
exec i3
XSESS
chmod +x "${ROOTFS_DIR}/root/.xsession"

# Create .bash_profile to auto-startx on tty1
cat > "${ROOTFS_DIR}/root/.bash_profile" << 'BASHPROF'
# AUTO-STARTX on tty1
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
    startx -- -nocursor 2>/dev/null
    logout
fi
BASHPROF