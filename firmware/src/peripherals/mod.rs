//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "mcu-esp32")]
#[path = "esp32.rs"]
pub mod ඞ;

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
pub mod ඞ;

#[cfg(feature = "mcu-nrf52840")]
#[path = "nrf52840.rs"]
pub mod ඞ;

/// Holds the peripherals. This merely exists to allow a way to pass around platform
/// specific peripherals, some of which may not even exist, in a platform-agnostic way.
pub struct Peripherals<I2c = (), Delay = (), Uart = (), UsbDriver = ()> {
	pub i2c: I2c,
	pub delay: Delay,
	pub uart: Uart,
	pub usb_driver: UsbDriver,
}
impl Peripherals {
	pub fn new() -> Self {
		Self {
			i2c: (),
			delay: (),
			uart: (),
			usb_driver: (),
		}
	}
}
impl<I2c, Delay, Uart, UsbDriver> Peripherals<I2c, Delay, Uart, UsbDriver> {
	#[allow(dead_code)]
	pub fn i2c<T>(self, p: T) -> Peripherals<T, Delay, Uart, UsbDriver> {
		Peripherals {
			i2c: p,
			delay: self.delay,
			uart: self.uart,
			usb_driver: self.usb_driver,
		}
	}
	#[allow(dead_code)]
	pub fn delay<T>(self, p: T) -> Peripherals<I2c, T, Uart, UsbDriver> {
		Peripherals {
			i2c: self.i2c,
			delay: p,
			uart: self.uart,
			usb_driver: self.usb_driver,
		}
	}
	#[allow(dead_code)]
	pub fn uart<T>(self, p: T) -> Peripherals<I2c, Delay, T, UsbDriver> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: p,
			usb_driver: self.usb_driver,
		}
	}
	#[allow(dead_code)]
	pub fn usb_driver<T>(self, p: T) -> Peripherals<I2c, Delay, Uart, T> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: self.uart,
			usb_driver: p,
		}
	}
}
