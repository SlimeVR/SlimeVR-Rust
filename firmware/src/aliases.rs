#[cfg(feature = "mcu-esp32c3")]
pub mod ඞ {
	pub use esp32c3_hal::ehal;
	pub use esp32c3_hal::Delay as DelayConcrete;

	pub type I2cConcrete = esp32c3_hal::i2c::I2C<esp32c3_hal::pac::I2C0>;
}

#[cfg(feature = "mcu-nrf52840")]
pub mod ඞ {
	pub use nrf52840_hal::Delay as DelayConcrete;

	pub type I2cConcrete = nrf52840_hal::Twim<nrf52840_hal::pac::TWIM0>;
}

pub trait I2c:
	embedded_hal::blocking::i2c::Write<Error = <Self as I2c>::Error>
	+ embedded_hal::blocking::i2c::WriteRead<Error = <Self as I2c>::Error>
{
	type Error: core::fmt::Debug;
}
impl<
		T: embedded_hal::blocking::i2c::Write<Error = E>
			+ embedded_hal::blocking::i2c::WriteRead<Error = E>,
		E: core::fmt::Debug,
	> I2c for T
{
	type Error = E;
}
