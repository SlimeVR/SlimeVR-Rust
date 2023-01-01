//! This module handles the typically platform-dependent setup of the peripherals

#[cfg(feature = "mcu-esp32")]
#[path = "esp32.rs"]
pub mod ඞ;

#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
pub mod ඞ;

#[cfg(mcu_f_nrf52)]
#[path = "nrf52.rs"]
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
/// Type-level builder for `Peripherals`, which transforms each field from () to the
/// peripheral type.
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

/// Type-level destructors for `Peripherals` which turn peripheral type into ().
impl<I2c, Delay, Uart, UsbDriver> Peripherals<I2c, Delay, Uart, UsbDriver> {
	#[cfg(all(bbq, feature = "log-usb-serial"))]
	pub fn bbq_peripheral(self) -> (UsbDriver, Peripherals<I2c, Delay, Uart, ()>) {
		(
			self.usb_driver,
			Peripherals {
				i2c: self.i2c,
				delay: self.delay,
				uart: self.uart,
				usb_driver: (),
			},
		)
	}
	#[cfg(all(bbq, feature = "log-uart"))]
	pub fn bbq_peripheral(self) -> (Uart, Peripherals<I2c, Delay, (), UsbDriver>) {
		(
			self.uart,
			Peripherals {
				i2c: self.i2c,
				delay: self.delay,
				uart: (),
				usb_driver: self.usb_driver,
			},
		)
	}
	#[cfg(not(bbq))]
	pub fn bbq_peripheral(self) -> ((), Peripherals<I2c, Delay, Uart, UsbDriver>) {
		((), self)
	}
}
