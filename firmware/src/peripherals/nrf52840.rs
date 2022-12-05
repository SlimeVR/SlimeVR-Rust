use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;
use crate::aliases::ඞ::UartConcrete;

use defmt::debug;
use embassy_nrf::interrupt;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::uarte::{self, Uarte};

pub fn get_peripherals(
) -> Peripherals<I2cConcrete<'static>, DelayConcrete, UartConcrete<'static>> {
	let p = embassy_nrf::init(Default::default());
	debug!("Initializing TWIM (I2C controller)");

	// IDK how this works, code is from here:
	// https://github.com/embassy-rs/embassy/blob/f109e73c6d7ef2ad93102b7c8223f5cef30ef36f/examples/nrf/src/bin/twim.rs
	let twim = {
		let config = twim::Config::default();
		let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
		Twim::new(p.TWISPI0, irq, p.P0_03, p.P0_04, config)
	};

	let delay = embassy_time::Delay;

	let uarte = {
		let irq = interrupt::take!(UARTE0_UART0);
		let mut config = uarte::Config::default();
		config.parity = uarte::Parity::EXCLUDED;
		config.baudrate = uarte::Baudrate::BAUD115200;
		let rx = p.P1_12;
		let tx = p.P1_11;
		Uarte::new(p.UARTE0, irq, rx, tx, config)
	};

	let p = Peripherals::new();
	p.i2c(twim).delay(delay).uart(uarte)
}
