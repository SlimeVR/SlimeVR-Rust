# Firmware
An implementation of SlimeVR firmware, built in Rust. Uses `embedded-hal` instead of
Arduino.

## Target Hardware
For now, we are only focused on the ESP32 platform, specifically the ESP32-C3.
This is because the ESP32-C3 is a RISC-V architecture, which is natively supported
by rust without needing a fork of llvm.

Furthermore, the ESP32-C3 supports bluetooth in additon to wifi, has hardware I2C,
built-in USB Serial and JTAG debugging.

The ESP32-C3 also claims to have a stronger wifi signal, which will help with lag.

## How to flash the firmware
### First time setup
1. Install cargo and rust via the instructions at https://rustup.rs
1. run `cargo install cargo-embed`

### Flashing
Simply run `cargo embed flash` from this folder to flash the firmware.
