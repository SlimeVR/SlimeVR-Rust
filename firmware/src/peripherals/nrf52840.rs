use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;

use defmt::debug;
use embassy_nrf::interrupt;
use embassy_nrf::twim::{self, Twim};

pub fn get_peripherals() -> Peripherals<I2cConcrete<'static>, DelayConcrete> {
	let p = embassy_nrf::init(Default::default());
	debug!("Initializing TWIM (I2C controller)");

	// IDK how this works, code is from here:
	// https://github.com/embassy-rs/embassy/blob/f109e73c6d7ef2ad93102b7c8223f5cef30ef36f/examples/nrf/src/bin/twim.rs
	let config = twim::Config::default();
	let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
	let twim = Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config);

	let delay = embassy_time::Delay;
	Peripherals { i2c: twim, delay }
}
