mod math;

use self::math::{discrete_to_radians, GyroFsr};
use crate::aliases::I2c;
use crate::imu::{FusedImu, Quat};
use crate::utils;

use ::bmi160::{AccelerometerPowerMode, GyroscopePowerMode, SensorSelector};
use defmt::{debug, trace};
use embedded_hal::blocking::delay::DelayMs;
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

		macro_rules! unwrap_or_err {
			($d:expr, $e:expr) => {
				match $e {
					Ok(v) => v,
					Err(err) => return Err(($d.destroy(), err)),
				}
			};
		}

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
				let id = unwrap_or_err!(driver, driver.chip_id());
				debug!("Constructed BMI with chip id: {}", id);
				unwrap_or_err!(
					driver,
					driver.set_accel_power_mode(AccelerometerPowerMode::Normal)
				);
				unwrap_or_err!(
					driver,
					driver.set_gyro_power_mode(GyroscopePowerMode::Normal)
				);
				debug!("BMI power mode set to Normal");
				delay.delay_ms(100);
				Ok(Self { driver })
			},
			|i| debug!("Retrying IMU connection (attempts so far: {})", i + 1),
		)
		// Map converts from tuple -> struct
		.map_err(|(i2c, error)| InitError { i2c, error })
	}
}

impl<I: I2c> FusedImu for Bmi160<I> {
	type Error = BmiError<I>;

	const IMU_TYPE: ImuType = ImuType::Bmi160;

	fn quat(&mut self) -> nb::Result<Quat, Self::Error> {
		let data = self.driver.data(SensorSelector::new().gyro())?;
		let gyro_vel_euler = data.gyro.unwrap();

		// TODO: We should probably query the IMU for the FSR instead of assuming the default one.
		const FSR: GyroFsr = GyroFsr::DEFAULT;

		// TODO: Check that bmi crates conventions for euler angles matches nalgebra.
		// TODO: Implement sensor fusion and temperature compensation
		// TODO: This should be integrated to position, lol
		Ok(nalgebra::UnitQuaternion::from_euler_angles(
			discrete_to_radians(FSR, gyro_vel_euler.x),
			discrete_to_radians(FSR, gyro_vel_euler.y),
			discrete_to_radians(FSR, gyro_vel_euler.z),
		))
	}
}

#[allow(dead_code)]
pub fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl DelayMs<u32>,
) -> impl crate::imu::FusedImu {
	Bmi160::new(i2c, delay).expect("Failed to initialize BMI160")
}
