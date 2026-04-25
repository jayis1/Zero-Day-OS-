# /etc/bash.bashrc — ZERO-DAY OS custom bashrc
# Sourced for all users

export PATH="/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
export TERM=xterm-256color
export EDITOR=nano
export PAGER=less
export LESS="-R"
export HISTSIZE=1000
export HISTFILESIZE=2000
export HISTCONTROL=ignorespace:erasedups
export HISTIGNORE="history*:panic:clear:exit"

if [ "$(id -u)" -eq 0 ]; then
    PROMPT='\[\033[01;31m\]┌──(\[\033[01;32m\]zeroday㉿\h\[\033[01;31m\])-[\[\033[01;33m\]\w\[\033[01;31m\]]\n\[\033[01;31m\]└─\[\033[01;32m\]#\[\033[00m\] '
else
    PROMPT='\[\033[01;32m\]┌──(\[\033[01;33m\]operator㉿\h\[\033[01;32m\])-[\[\033[01;34m\]\w\[\033[01;32m\]]\n\[\033[01;32m\]└─\[\033[01;33m\]\$\[\033[00m\] '
fi

PS1="$PROMPT"

alias ll='ls -la --color=auto'
alias la='ls -A --color=auto'
alias l='ls -CF --color=auto'
alias grep='grep --color=auto'
alias fgrep='fgrep --color=auto'
alias egrep='egrep --color=auto'
alias ..='cd ..'
alias ...='cd ../..'
alias cls='clear'
alias h='history'
alias ports='ss -tulanp'
alias myip='ip -4 addr show | grep inet'
alias battery='cardputer-battery'
alias wifi='cardputer-wifi-toggle'
alias dongle='dongle-setup status'
alias stealth='power-mode stealth'
alias perf='power-mode performance'
alias bal='power-mode balanced'

zeroday_info() {
    if [ -f /etc/zeroday-release ]; then
        cat /etc/zeroday-release
        echo ""
    fi
    echo "Kernel: $(uname -r)"
    echo "Uptime: $(uptime -p)"
    echo "Load: $(cat /proc/loadavg | awk '{print $1, $2, $3}')"
    echo "RAM: $(free -m | awk '/Mem:/{printf "%dM/%dM", $3, $2}')"
    echo "Disk: $(df -h / | awk 'NR==2{print $3"/"$2" ("$5")"}')"
    echo "WiFi: $(iw dev wlan0 info 2>/dev/null | awk '/type/{print $2}' || echo 'N/A')"
    if ip link show wlan1 &>/dev/null; then
        echo "Dongle: $(iw dev wlan1 info 2>/dev/null | awk '/type/{print $2}' || echo 'present')"
    else
        echo "Dongle: not connected"
    fi
    cardputer-battery --short 2>/dev/null || true
}

if [ -f /etc/zeroday-motd ]; then
    cat /etc/zeroday-motd
fi