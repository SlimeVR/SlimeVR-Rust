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
pub struct Peripherals<I2c = (), Delay = (), Uart = (), UsbDriver = (), Net = ()> {
	pub i2c: I2c,
	pub delay: Delay,
	pub uart: Uart,
	pub usb_driver: UsbDriver,
	pub net: Net,
}
impl Peripherals {
	pub fn new() -> Self {
		Self {
			i2c: (),
			delay: (),
			uart: (),
			usb_driver: (),
			net: (),
		}
	}
}
/// Type-level builder for `Peripherals`, which transforms each field from () to the
/// peripheral type.
impl<I2c, Delay, Uart, UsbDriver, Net> Peripherals<I2c, Delay, Uart, UsbDriver, Net> {
	#[allow(dead_code)]
	pub fn i2c<T>(self, p: T) -> Peripherals<T, Delay, Uart, UsbDriver, Net> {
		Peripherals {
			i2c: p,
			delay: self.delay,
			uart: self.uart,
			usb_driver: self.usb_driver,
			net: self.net,
		}
	}
	#[allow(dead_code)]
	pub fn delay<T>(self, p: T) -> Peripherals<I2c, T, Uart, UsbDriver, Net> {
		Peripherals {
			i2c: self.i2c,
			delay: p,
			uart: self.uart,
			usb_driver: self.usb_driver,
			net: self.net,
		}
	}
	#[allow(dead_code)]
	pub fn uart<T>(self, p: T) -> Peripherals<I2c, Delay, T, UsbDriver, Net> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: p,
			usb_driver: self.usb_driver,
			net: self.net,
		}
	}
	#[allow(dead_code)]
	pub fn usb_driver<T>(self, p: T) -> Peripherals<I2c, Delay, Uart, T, Net> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: self.uart,
			usb_driver: p,
			net: self.net,
		}
	}
	#[allow(dead_code)]
	pub fn net<T>(self, p: T) -> Peripherals<I2c, Delay, Uart, UsbDriver, T> {
		Peripherals {
			i2c: self.i2c,
			delay: self.delay,
			uart: self.uart,
			usb_driver: self.usb_driver,
			net: p,
		}
	}
}

/// Type-level destructors for `Peripherals` which turn peripheral type into ().
impl<I2c, Delay, Uart, UsbDriver, Net> Peripherals<I2c, Delay, Uart, UsbDriver, Net> {
	#[cfg(all(bbq, feature = "log-usb-serial"))]
	pub fn bbq_peripheral(self) -> (UsbDriver, Peripherals<I2c, Delay, Uart, (), Net>) {
		(
			self.usb_driver,
			Peripherals {
				i2c: self.i2c,
				delay: self.delay,
				uart: self.uart,
				usb_driver: (),
				net: self.net,
			},
		)
	}
	#[cfg(all(bbq, feature = "log-uart"))]
	pub fn bbq_peripheral(self) -> (Uart, Peripherals<I2c, Delay, (), UsbDriver, Net>) {
		(
			self.uart,
			Peripherals {
				i2c: self.i2c,
				delay: self.delay,
				uart: (),
				usb_driver: self.usb_driver,
				net: self.net,
			},
		)
	}
	#[cfg(not(bbq))]
	pub fn bbq_peripheral(self) -> ((), Peripherals<I2c, Delay, Uart, UsbDriver, Net>) {
		((), self)
	}
}
