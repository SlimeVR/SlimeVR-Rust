# Memory Layout

# NRF52
The nrf52 series of chips uses softdevice, which is a closed source BLE stack provided
by the manufacturer. Softdevice has different versions, and each board tends to come
with a different version already flashed.

It is very important that you choose the right memory layout, otherwise the rust
firmware won't boot, and even worse, **it may overwrite whatever softdevice the board
vendor flashed to your chip!** If that happens, you'll need to restore your softdevice,
see below.

## Finding your softdevice version
Most commonly, boards ship with a UF2 bootloader pre-flashed. The UF2 bootloader will
appear as a USB drive when you enter bootloader mode, often by double tapping the reset
or by holding down a boot button while resetting.

In this case, usually the boards have a `INFO_UF2.TXT` file, which may look like this:
```
UF2 Bootloader 0.6.1 lib/nrfx (v2.0.0) lib/tinyusb (0.10.1-293-gaf8e5a90) lib/uf2 (remotes/origin/configupdate-9-gadbb8c7)
Model: Seeed XIAO nRF52840
Board-ID: Seeed_XIAO_nRF52840_Sense
SoftDevice: S140 version 7.3.0
Date: Nov 12 2021
```
You can see here that the softdevice version is `S140 v7.3.0`.

## Configuring the firmware for your softdevice
You should now know your softdevice version, for example `S140 v7.3.0`. If you don't,
reread [## Finding your softdevice version] or try [## Restoring the softdevice].

Check under the `linker_scripts/` folder to see if we already figured out how to get
your softdevice version working. If not, you'll have to create your own file, and should
also open a PR once you get it working to help everyone else out. 

### Configuring your own memory.x file
Now, we need to consult the nrf52 docs to find out what the appropriate values for where
to put the code in FLASH and RAM.

Nordic documents the [memory layout] as having two important parameters, `APP_CODE_BASE`
and `APP_RAM_BASE`. Unfortunately these values are not easy to find, so we have to
compute them ourselves.

Check 

## Restoring the softdevice
If you are not sure what your softdevice version is, or if you accidentally mangled or
erased it, the safest course of action is to reflash the bootloader + softdevice.

### Common bootloaders
Generally board manufacturers will provide bootloaders for you, here are some common
ones.

Many use the [adafruit bootloader][Adafruit], if they do, check the Assets under
the latest release, and download the `.hex` file that corresponds to your board. The
file name will be `boardname-BOOTLOADER-sXXX_VERSION.hex`. for example,
`nice_nano_bootloader-0.7.0_s140_6.1.1.hex`. The `SXXX-VERSION` part is the important
part, each softdevice version will have a different memory layout, so the example above
is softdevice S140 v6.1.1.

- Seeed studio XIAO: the wiki has a section on [power verification][xiao wiki]
  which links to the bootloader [here][xiao bootloader]
- nice!nano: [adafruit bootloader][adafruit]

### Flashing the bootloader.
Once you have the .hex file, you need to determine if your bootloader is working or not.
If your device appears as a USB drive your bootloader should be working, so try dragging
the hex file onto the USB drive. That should reflash it. It should disapper from USB
when you do that, if its done correctly.

If it shows up as USB but the hex file doesn't do anything when you drag it on, your
bootloader is either broken, or rejecting the new upgrade because of a mismatched family
ID. Your best course of action is to reflash with [probe-rs] or search our issue tracker
on github for that board. If no one else solved it already, file an issue and we will
try to help!

If you have already configured `probe-rs` you can run the following command:
```
probe-rs-cli download --chip nrf52840 --chip-erase --format hex path/to/booloader.hex 
```

If it worked, you should see it slowly flashing the device, and then you should be able
to appear as USB when reset into bootloader mode. If not, you either used `probe-rs`
incorrectly, or your hardware is fried. Feel free to open an issue on github, or ask
on discord.



[adafruit]: https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases/
[probe-rs]: /docs/Debugging.md
[xiao wiki]: https://wiki.seeedstudio.com/XIAO_BLE/#power-consumption-verification
[xiao bootloader]: https://github.com/0hotpotman0/BLE_52840_Core/tree/main/bootloader
[memory layout]: https://infocenter.nordicsemi.com/index.jsp?topic=%2Fsds_s140%2FSDS%2Fs1xx%2Fmem_usage%2Fmem_resource_map_usage.html
