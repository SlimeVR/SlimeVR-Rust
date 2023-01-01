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

Check the `build.rs` build script to see the list of known softdevice versions. If you
don't see yours listed, you'll have to configure the values yourself, and should also
open a PR once you get it working to help everyone else out. 

### Configuring `build.rs`
If your softdevice is not listed in `build.rs`, we need to consult the nrf52 docs to
find out the appropriate values for the `SoftdeviceInfo` struct.

Nordic documents the [memory layout] as having two important parameters, `APP_CODE_BASE`
and `APP_RAM_BASE`. Unfortunately these values are computed, so we have to compute them
ourselves. The main one we care about is `APP_CODE_BASE` which will determine where we
put our code in flash.

As the documentation describes, we can compute `APP_CODE_BASE` like this:
`APP_CODE_BASE = SD_FLASH_SIZE + MBR_SIZE`. You can search for [`SD_FLASH_SIZE`] and
[`MBR_SIZE`] in the nordic docs to get their values for your softdevice version. For
S140 v7.3.0, this is `0x26000+0x1000 = 0x27000`. So the struct in `build.rs`

`APP_RAM_BASE` aka `RAM` can be set to `0x20000000 + SD_RAM_SIZE`, and when we run the
firmware, we where `SD_RAM_SIZE` is the size of the softdevice's ram usage. You can
set this to `0x0` for now and it will print an error to the logger on boot telling you
what the correct value is.

So for example, for softdevice S140, we added a softdevice const in `build.rs`:
```rust
const S140: SoftdeviceInfo = SoftdeviceInfo {
  sd_flash_size: 0x26000,
  mbr_size: 0x1000,
  sd_ram_size: 0x0 // Default to 0x0 if you're not sure
}
```

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
[`SD_FLASH_SIZE`]: https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.s140.api.v7.3.0/group___n_r_f___s_d_m___d_e_f_i_n_e_s.html#gab12c6fdc9b9756dc76a8888172ef9a0b
[`MBR_SIZE`]: https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.s140.api.v7.3.0/group___n_r_f___m_b_r___d_e_f_i_n_e_s.html#ga2f71568a2395dc0783c1e6142ef71d5b
