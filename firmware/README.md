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

## Prerequesites
In addition to regular rust tools (which can be installed via rustup), you should run
`cargo install espflash` to install the utility that flashes the esp.
