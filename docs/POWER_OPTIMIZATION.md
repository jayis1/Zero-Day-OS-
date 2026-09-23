# ZERO-DAY OS — Power Optimization Layer

Target: CardPuterZero (RP3A0 / BCM2837, aarch64, 512MB RAM, BQ27220 fuel gauge)

## Overview

Power optimization for CardPuterZero is delivered in three layers:

1. **zd-power** — power management CLI (profiles, boot tune, battery watch daemon)
2. **zeroday-power.service** — systemd unit that runs at boot before zeroday-boot
3. **zd-boot-profile** — boot time profiler and optimization advisor

---

## Power Profiles

| Profile     | Governor   | Max Freq | Cores | Radios         | Backlight | Est. Life |
|-------------|------------|----------|-------|----------------|-----------|-----------|
| performance | performance| 1 GHz    | 4     | All on         | 100%      | ~4h       |
| balanced    | ondemand   | 800 MHz  | 2     | WiFi on, BT off| 63%       | ~6h       |
| stealth     | powersave  | 600 MHz  | 1     | All blocked    | 31%       | ~10h      |
| ultra       | powersave  | 300 MHz  | 1     | All blocked    | 16%       | ~14h      |
| auto        | adaptive   | adaptive | adaptive | adaptive  | adaptive  | varies    |

**auto** profile selects based on battery level:
- ≥60%: balanced
- 30-59%: stealth
- <30%: ultra
- Charging: balanced

---

## Boot-time Tuning (zd-power tune)

Applied once at boot by `zeroday-power.service`:

- VM dirty ratio: 5% / 2% (reduce SD write storms)
- Swappiness: 10 (prefer RAM, avoid SD I/O)
- USB autosuspend: 2s for all devices
- SD/MMC runtime PM: auto
- I/O scheduler: deadline/mq-deadline on mmcblk
- Read-ahead: 128KB on mmcblk (down from default 512KB)
- CPU idle states: enabled (deeper C-states)

---

## Commands

```
zd-power status                  # Show current power state
zd-power profile auto            # Auto-select by battery
zd-power profile performance     # Full throttle
zd-power profile balanced        # Daily driver
zd-power profile stealth         # Extended battery, no RF
zd-power profile ultra           # Maximum battery life
zd-power tune                    # Apply kernel power knobs
zd-power watch                   # Battery-adaptive daemon
zd-boot-profile                  # Show boot timing + recommendations
zd-boot-profile --verbose        # Include critical-chain and memory
```

---

## Boot Path Integration

`zeroday-power.service` runs **before** `zeroday-boot.service` and network targets
(ordered with `Before=zeroday-boot.service network.target`). This ensures:

1. Kernel power knobs are set before any boot services start
2. Battery-adaptive profile is applied before network, display, and service startup
3. No race between power mode and radio/display initialization

`zeroday-boot` delegates to `zd-power` instead of writing sysfs directly, so the
power manager is the single source of truth for CPU governor, frequency, and cores.

---

## Battery Watch Daemon

For workloads that need continuous adaptation, run the watch daemon:

```
systemctl start zeroday-power-watch.service   # (or: zd-power watch)
```

The daemon re-evaluates battery level every 60 seconds and switches profile
automatically when the battery crosses the 60% / 30% thresholds.

To enable persistently:
```
cp configs/systemd/zeroday-power-watch.service /etc/systemd/system/
systemctl enable --now zeroday-power-watch.service
```

---

## Files

| Path | Purpose |
|------|---------|
| `scripts/system/zd-power` | Power management CLI |
| `scripts/system/zd-boot-profile` | Boot time profiler |
| `configs/systemd/zeroday-power.service` | Boot-time systemd unit |
| `pi-gen/stage3/03-boot-scripts/01-run.sh` | Installs and enables at image build time |
| `docs/POWER_OPTIMIZATION.md` | This document |
