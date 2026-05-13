#!/bin/bash
set -e
echo "=== Pushing to GitHub ==="
git push -u origin main
echo "=== Pushing tag v1.5.0 ==="
git push origin v1.5.0
echo "=== Creating GitHub Release ==="
gh release create v1.5.0 \
  --title "v1.5.0 — aarch64 Cardputer Zero" \
  --notes "$(cat <<'EOF'
## ZERO-DAY OS v1.5.0 — M5Stack Cardputer Zero

### Highlights
- **Architecture**: arm64/aarch64 for RP3A0 SoC (Pi Zero 2W die), 512MB LPDDR2
- **Official DT overlay**: `cardputerzero-overlay.dts` — ST7789V 320×170 LCD, TCA8418 46-key keyboard, ES8390 audio, BMI270 IMU, RX8130 RTC, PY32IO16 I/O expander
- **Dual UI**: Xorg+i3+Textual TUI (pen-test mode) / labwc+LVGL (official mode)
- **92 functional scripts**: WiFi attacks, BT scanning, NFC, SDR/sub-GHz, mesh, IR, camera, reverse shells, hardware probes
- **pi-gen Docker build**: Full aarch64 cross-compilation with qemu-user-static

### Hardware Support
| Component | Driver/Overlay |
|-----------|---------------|
| ST7789V LCD (320×170) | `sitronix,st7789v_m5stack` via SPI |
| TCA8418 Keyboard (5×10) | `m5stack,tca8418c` via I2C |
| ES8390 Audio Codec | `everest,es8390` via I2S |
| BMI270 IMU | `bosch,bmi270` via I2C |
| RX8130 RTC | `epson,rx8130` via I2C |
| PY32IO16 I/O Expander | `m5stack,py32io16` via I2C |

### Bootable Images
| Image | Contents |
|-------|----------|
| `zeroday-os-.zip` | Stage 2 base (Debian minimal) |
| `zeroday-os--zeroday.zip` | Stage 3 (Xorg+i3+hardware) |
| `zeroday-os--zeroday-lite.zip` | Stage 4 (pen-test tools) |
| `zeroday-os--zeroday-full.zip` | Stage 5 (complete system) |

### Flash
```bash
unzip 2026-05-13-zeroday-os--zeroday-full.zip
sudo dd if=2026-05-13-zeroday-os--zeroday-full.img of=/dev/sdX bs=4M status=progress conv=fsync
```

### Known Limitations
- 512MB RAM — no metasploit (OOM risk)
- OpenCode aarch64 binary not available yet; stub placeholder installed
- pip packages (nfcpy, cc1101, rfcat, meshtastic) deferred to first boot
EOF
)"
echo "=== Done! ==="
