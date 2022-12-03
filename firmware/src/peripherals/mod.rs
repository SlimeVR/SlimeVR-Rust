//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
pub mod ඞ;

#[cfg(feature = "mcu-nrf52840")]
#[path = "nrf52840.rs"]
pub mod ඞ;

use crate::aliases::I2c;

pub struct Peripherals<I: I2c, D: embedded_hal::blocking::delay::DelayMs<u32>> {
	pub i2c: I,
	pub delay: D,
}
