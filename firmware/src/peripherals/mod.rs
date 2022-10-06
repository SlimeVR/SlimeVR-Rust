//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
mod ඞ;

pub use ඞ::get_peripherals;

use crate::aliases::I2c;

pub struct Peripherals<I: I2c, D: embedded_hal::blocking::delay::DelayMs<u32>> {
    pub i2c: I,
    pub delay: D,
}
