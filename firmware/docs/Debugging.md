# Debugging the code
Help us update this document! Its very out of date. A lot of the stuff in here is
misleading or no longer relevant.

## Tools used
### Cargo Embed
To make things easy, we have set things up to use [Cargo Embed].
This provides flashing, log/RTT monitoring, and debugging all in one!

Simply run:
```
cargo embed
```
This will flash the device and show you the log output. You can also do `cargo embed monitor`
if you don't want to flash and merely want to view logs/debug.

### Defmt
We use [`defmt`] as the logging API. It is more efficient than the `log` crate as it
doesn't need to actually allocate strings or serialize the arguments into a string.

We also use `defmt-rtt` to use RTT as the mechanism to send these logs to the host
computer.
> **Note**:
> RTT is a communication protocol uses in-memory ringbuffers as the place to store the
> data to be "printed", and then a hardware debugger reads those memory regions. This
> lets us log out data without having to do slow serial IO.
>
> In the case of the esp32c3, we don't need any additional hardware probes to get
> hardware debugging working, as it has built-in JTAG support over USB.


### Hardware debugging / GDB remote debugging
The ESP32-C3 supports JTAG over USB, so you can actually set breakpoints and debug
without needing any additional hardware. This also doesn't require any special
software support, unlike the ESP8266.

To accomplish this, `cargo embed` starts a GDB server that you can connect to at
`127.0.0.1:1337`. You can then connect to that server using GDB, either via the command
line or via your IDE.

Please note that there is currently a [bug][gdb rtt bug] that prevents RTT and GDB from
being used at the same time, you will probably need to turn off RTT in
[`Embed.toml`](/Embed.toml)

#### Installing GDB
First install GDB. Follow the normal process for this, you may have to google it.
> **Note**: If you are on a new macos computer that uses ARM64 (apple m1 chip or later),
> you might not be able to install gdb through `brew`. Espressif provides a
> [mac gdb][espressif gdb] build that you can use. Just add that to your `PATH`.

#### Connecting to the remote gdb server
You then need use the gdb client (the gdb executable you just installed) to connect to
the gdb server that `cargo embed` is running.

You can do this on the command line or via your IDE.

##### With the command line
Follow [these instructions](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-guides/jtag-debugging/using-debugger.html#jtag-debugging-using-debugger-command-line)

##### With VSCode
I have added a [`launch.json`](/.vscode/launch.json) file for you already, so you can probably just use that.
If that doesn't work, make sure that gdb is in your PATH and do some googling. I'm new
to this too!

[`defmt`]: https://defmt.ferrous-systems.com
[espressif gdb]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/tools/idf-tools.html#riscv32-esp-elf-gdb
[gdb rtt bug]: https://github.com/probe-rs/probe-rs/issues/1221
[Cargo Embed]: https://probe.rs/docs/tools/cargo-embed/

