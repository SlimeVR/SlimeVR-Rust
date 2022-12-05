//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
pub mod ඞ;

#[cfg(feature = "mcu-nrf52840")]
#[path = "nrf52840.rs"]
pub mod ඞ;

/// Holds the peripherals. This merely exists to allow a way to pass around platform
/// specific peripherals, some of which may not even exist, in a platform-agnostic way.
pub struct Peripherals<I2c = (), Delay = (), Uart = ()> {
	pub i2c: I2c,
	pub delay: Delay,
	pub uart: Uart,
}
impl Peripherals {
	pub fn new() -> Self {
		Self {
			i2c: (),
			delay: (),
			uart: (),
		}
	}
}
impl<I2c, Delay, Uart> Peripherals<I2c, Delay, Uart> {
	#[allow(dead_code)]
	pub fn i2c<N>(self, p: N) -> Peripherals<N, Delay, Uart> {
		Peripherals {
			i2c: p,
			delay: self.delay,
			uart: self.uart,
		}
	}
	#[allow(dead_code)]
	pub fn delay<N>(self, p: N) -> Peripherals<I2c, N, Uart> {
		Peripherals {
			i2c: self.i2c,
			delay: p,
			uart: self.uart,
		}
	}
	#[allow(dead_code)]
	pub fn uart<N>(self, p: N) -> Peripherals<I2c, Delay, N> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: p,
		}
	}
}
