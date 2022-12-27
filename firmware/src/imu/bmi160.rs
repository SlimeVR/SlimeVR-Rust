use super::{Imu, ImuKind, Quat};
use crate::aliases::I2c;
use crate::utils;

use defmt::{debug, trace};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use firmware_protocol::ImuType;

type BmiDriver<I2c> = ::bmi160::Bmi160<bmi160::interface::I2cInterface<I2c>>;
// Second generic is `()` because we don't have chip select errors in I2C.
type BmiError<I> = ::bmi160::Error<<I as I2c>::Error, ()>;

pub struct InitError<I: I2c> {
	pub i2c: I,
	pub error: BmiError<I>,
}
impl<I> core::fmt::Debug for InitError<I>
where
	I: I2c,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		self.error.fmt(f)
	}
}

pub struct Bmi160<I: I2c> {
	driver: BmiDriver<I>,
}
impl<I: I2c> Bmi160<I> {
	pub fn new(i2c: I, delay: &mut impl DelayMs<u32>) -> Result<Self, InitError<I>> {
		debug!("Constructing BMI160...");
		let addr = ::bmi160::SlaveAddr::Default;
		debug!("I2C address: {:?}", defmt::Debug2Format(&addr));

		utils::retry(
			4,
			i2c,
			|mut i2c| {
				delay.delay_ms(100);
				trace!("Flushing I2C with bogus data");
				let _ = i2c.write(addr.addr(), &[0]);
				delay.delay_ms(100);
				trace!("Constructing IMU");
				let mut driver = BmiDriver::new_with_i2c(i2c, addr);
				let id = match driver.chip_id() {
					Ok(id) => id,
					Err(error) => return Err((driver.destroy(), error)),
				};
				debug!("Constructed BMI with chip id: {}", id);
				// TODO: Do we need to enable rotation or do any other setup?
				delay.delay_ms(100);
				Ok(Self { driver })
			},
			|i| debug!("Retrying IMU connection (attempts so far: {})", i + 1),
		)
		// Map converts from tuple -> struct
		.map_err(|(i2c, error)| InitError { i2c, error })
	}
}

impl<I: I2c> Imu for Bmi160<I> {
	type Error = BmiError<I>;

	const IMU_TYPE: ImuType = ImuType::Bmi160;

	fn quat(&mut self) -> nb::Result<Quat, Self::Error> {
		todo!()
	}
}

pub fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl DelayMs<u32>,
) -> impl crate::imu::Imu {
	Bmi160::new(i2c, delay).expect("Failed to initialize BMI160")
}
