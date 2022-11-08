# NRF Demo
This code demos the use of the nrf52840 on embedded rust.

## Prerequisites
You need all the usual rust tools, which can be gotten from [rustup](https://rustup.rs).
Then, we need the `flip-link` linker which improves memory safety. Install it via
`cargo install flip-link`. We also need to `cargo install cargo-binutils` to get
objcopy. Finally we need to `cargo install uf2conv` to allow us to convert from .bin to
the uf2 flashing format.

## Building the code
You can check that code compiles with `cargo check` or `cargo build`. 

To actually build the code, you should run `cargo objcopy -- -O binary nrf_demo.bin` to
get the .bin file for the firmware (note that this will run `cargo build` for you).
Then we need to convert it into uf2 format by running `uf2conv nrf_demo.bin -b 0x26000
--family 0xada52840`.

Then you can double tap the reset button (or short RST to GND) to get into bootloader
mode. After doing so, a USB drive for the NRF will appear, into which you may copy your
UF2 file you created. This will upload and flash the firmware to the nrf.

Please note that this assumes your NRF chip has the softdevice firmware already loaded
on the bootloader, and that the bootloader starts the code at 0x2600. If not, you may
need to use `-b 0x1000` instead and modify `memory.x`. We found that the seed studio
bootloader did not work with this approach, so we had to flash the adafruit bootloader
on instead.
