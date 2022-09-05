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

## How to flash/debug the firmware

### Prerequesites
In addition to regular rust tools (which can be installed via rustup), you should run
`cargo install cargo-embed` to install the utility that flashes the esp.

### Flashing the firmware and viewing logs
To make things easy, we have set things up to use [Cargo Embed].
This provides flashing, log/RTT monitoring, and debugging all in one!

Simply run:
```
cargo embed
```
This will flash the device and show you the log output. You can also do `cargo embed monitor`
if you don't want to flash.

[Cargo Embed]: https://probe.rs/docs/tools/cargo-embed/

#### Defmt
We use [`defmt`] as the logging API. It is more efficient than the `log` crate as it
doesn't need to actually allocate strings or serialize the arguments of those strings.

We also use `defmt-rtt` to use RTT as the mechanism to send these logs to the host
computer.
> **Note**:
> RTT uses in-memory ringbuffers as the place to store the data to be "printed", and
> then the debugger reads those memory regions. This lets us log out data without
> having to do slow serial IO.

[`defmt`]: https://defmt.ferrous-systems.com

### Hardware debugging / GDB remote debugging
The ESP32-C3 supports JTAG over USB, so you can actually set breakpoints and debug
without needing any additional hardware. This also doesn't require any special
software support, unlike the ESP8266.

To accomplish this, `cargo embed` starts a GDB server that you can connect to at
`127.0.0.1:1337`. To actually use this with GDB/LLDB's CLI or your IDE is left as
an excercise to the reader (I havent learned gdb remote debugging yet).

If you know how to do this, [help me out](https://github.com/SlimeVR/SlimeVR-Rust/issues/31)!
