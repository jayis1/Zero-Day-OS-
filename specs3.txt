# M5Stack Cardputer ZERO: Complete Master Specifications

This document synthesizes all available information from the device schematics, specification sheets, and the official M5Stack launch campaign.

## 1. Overview & Device Concept
The M5Stack CardputerZero is marketed as a "Pocket Linux Lab" and the "Ultimate Hacking Toolkit". 
It packs real Linux capability, CLI tools, hardware expansion, and Edge AI into a credit-card sized form factor.

*   **Form Factor:** Credit-Card Size (85x54mm)
*   **All-in-One Design:** Built-in 46-key matrix keyboard, display, internal battery, microphone, and speaker.
*   **OS/Software Ecosystem:** Powered by the Raspberry Pi CM0, running Linux. It features an on-device app store allowing users to switch between apps and community firmware instantly—without a PC or reflashing.
*   **Edge AI:** Runs lightweight edge AI tools like **OpenClaw** for portable assistance, automation, and testing.

## 2. Core Hardware Specifications
*   **Processor:** Raspberry Pi Compute Module 0 (CM0) - Broadcom BCM2837, Quad-Core Cortex-A53 1 GHz
*   **Memory & Storage:** 512 MB LPDDR2 RAM, microSD card slot for storage expansion
*   **Wireless Connectivity:** 2.4GHz Wi-Fi (802.11 b/g/n), Bluetooth 4.2 / BLE
*   **Display:** 1.9" LCD (ST7789v3), plus HDMI output supporting 1080P at 30fps
*   **Camera:** Sony IMX219, 8MP (3280 x 2464 resolution), CSI 4-Lane interface
*   **Input:** 46-key matrix keyboard (Built-in)
*   **Networking:** 10/100M Ethernet (LAN port)
*   **Video Codec:** Decode: H.264 / MPEG-4 @ 1080P 30fps | Encode: H.264 @ 1080P 30fps
*   **USB 2.0:** Host / Slave switchable, 2x USB Type-C, 1x USB-A
*   **Battery & Power:** 3.7V / 1500 mAh Li-Po internal battery, managed by BQ27220 fuel gauge
*   **Audio:** ES8389 audio codec, built-in MEMS microphone, 1W internal speaker, 3.5mm TRS audio output jack
*   **Sensors:** BMI270 IMU (gyroscope + accelerometer), RX8130CE RTC (Real-Time Clock)
*   **Expansion Interfaces:** 
    *   HY2.0-4P port (Grove - I2C / UART switchable)
    *   2.54-14P ExtPort bus (SPI, UART, I2C, USB, GPIO, 5V)
*   **IR Transceiver:** Built-in Infrared TX & RX

## 3. Pricing & Launch Timeline
*   **CardputerZero Lite:** $59 (Super Early Bird) / $99 (MSRP)
*   **CardputerZero:** $89 (Super Early Bird) / $149 (MSRP)
*   **Launch Date:** Planned Kickstarter launch in **mid-to-late May**. Expected to run for one month.

## 4. Primary Use Cases
*   **CLI & Edge AI:** Portable SSH, Python execution, and Git/Vim editing.
*   **Cybersecurity & Hacking:** Run security software and connect add-ons like CC1101 (Sub-GHz) and NFC modules for sniffing and testing.
*   **Off-Grid Communication:** Pairs with tools like Meshtastic for localized networking.
*   **Hardware Tinkering:** Works with 100+ M5Units via Grove and ExtPort for DIY prototyping.
*   **Portable Entertainment:** Retro gaming using physical buttons, and media consumption via the 3.5mm jack or speaker.

## 5. Detailed Board Layout (PCB Components)
### Front Layout Components
TYPE-C HOST, MIC, USB2 LAN, LCD screen, DC/DC converter, TF CARD (microSD) slot, KEYBOARD SCANNER, EIO, RTC, IMU, BATTERY METER, CHG IC, USB SW, POWER SWITCH, TYPE-C CHARGE port, AUDIO ICs, KEY MATRIX.

### Back Layout Components
GROVE connector, 3.5mm AUDIO OUTPUT, HDMI OUTPUT, SPEAKER PORT, BATTERY PORT, 2x7 SOCKET @2.54mm, CM0 module socket, USB-HUB, CAMERA PORT, USB-A HOST, BOOT BTN, 100M LAN port.
