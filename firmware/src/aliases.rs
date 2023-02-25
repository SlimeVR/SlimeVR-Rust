#![allow(unused)]

#[cfg(feature = "mcu-esp32")]
pub mod ඞ {
	pub use esp32_hal::ehal;
	pub use esp32_hal::Delay as DelayConcrete;

	pub type I2cConcrete<'a> = esp32_hal::i2c::I2C<'a, esp32_hal::peripherals::I2C0>;

	pub type BbqPeripheral<'a> = ();
}

#[cfg(feature = "mcu-esp32c3")]
pub mod ඞ {
	pub use esp32c3_hal::ehal;
	pub use esp32c3_hal::Delay as DelayConcrete;

	pub type I2cConcrete<'a> =
		esp32c3_hal::i2c::I2C<'a, esp32c3_hal::peripherals::I2C0>;

	pub type BbqPeripheral<'a> = ();
}

#[cfg(mcu_f_nrf52)]
pub mod ඞ {
	pub use embassy_time::Delay as DelayConcrete;

	pub type I2cConcrete<'a> =
		embassy_nrf::twim::Twim<'a, embassy_nrf::peripherals::TWISPI0>;

	pub type UartConcrete<'a> =
		embassy_nrf::uarte::Uarte<'a, embassy_nrf::peripherals::UARTE0>;

	#[cfg(feature = "mcu-nrf52840")]
	pub type UsbDriverConcrete<'a> = embassy_nrf::usb::Driver<
		'a,
		embassy_nrf::peripherals::USBD,
		embassy_nrf::usb::PowerUsb,
	>;

	#[cfg(feature = "mcu-nrf52832")]
	pub type UsbDriverConcrete<'a> = ();

	#[cfg(all(bbq, feature = "log-usb-serial"))]
	pub type BbqPeripheralConcrete<'a> = UsbDriverConcrete<'a>;
	#[cfg(all(bbq, feature = "log-uart"))]
	pub type BbqPeripheralConcrete<'a> = UartConcrete<'a>;
	#[cfg(not(bbq))]
	pub type BbqPeripheralConcrete<'a> = ();
}

pub trait I2c:
	embedded_hal::blocking::i2c::Write<Error = <Self as I2c>::Error>
	+ embedded_hal::blocking::i2c::WriteRead<Error = <Self as I2c>::Error>
	+ embedded_hal::blocking::i2c::Read<Error = <Self as I2c>::Error>
{
	type Error: core::fmt::Debug;
}
impl<
		T: embedded_hal::blocking::i2c::Write<Error = E>
			+ embedded_hal::blocking::i2c::WriteRead<Error = E>
			+ embedded_hal::blocking::i2c::Read<Error = E>,
		E: core::fmt::Debug,
	> I2c for T
{
	type Error = E;
}

pub trait Delay:
	embedded_hal::blocking::delay::DelayMs<u8> + embedded_hal::blocking::delay::DelayMs<u32>
{
}
impl<
		T: embedded_hal::blocking::delay::DelayMs<u8>
			+ embedded_hal::blocking::delay::DelayMs<u32>,
	> Delay for T
{
}
