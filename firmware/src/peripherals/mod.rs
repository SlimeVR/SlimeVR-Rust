//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "esp32c3")]
#[path = "esp32c3.rs"]
mod ඞ;

pub use ඞ::get_peripherals;

use crate::aliases::{ehal, I2c};

pub struct Peripherals<I: I2c, D: ehal::blocking::delay::DelayMs<u32>> {
    pub i2c: I,
    pub delay: D,
}
