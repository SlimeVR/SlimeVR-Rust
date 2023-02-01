mod math;

use self::math::GyroFsr;
use crate::aliases::I2c;
use crate::imu::drivers::bmi160::math::AccelFsr;
use crate::imu::fusion::new_fuser;
use crate::imu::{FusedData, FusedImu, Imu, UnfusedData};
use crate::utils;

use ::bmi160::{AccelerometerPowerMode, GyroscopePowerMode, SensorSelector};
use defmt::{debug, trace};
use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;
use nalgebra::vector;

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
		let addr = ::bmi160::SlaveAddr::Alternative(false);
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

impl<I: I2c> Imu for Bmi160<I> {
	type Error = BmiError<I>;
	type Data = UnfusedData;

	const IMU_TYPE: ImuType = ImuType::Bmi160;

	async fn next_data(&mut self) -> Result<Self::Data, Self::Error> {
		// Avoids permablocking async tasks, since we don't do any actual waiting.
		yield_now().await;

		let data = self.driver.data(SensorSelector::new().gyro().accel())?;
		let gyro = data.gyro.unwrap();
		let accel = data.accel.unwrap();

		// TODO: We should probably query the IMU for the FSR instead of assuming the default one.
		const GYRO_FSR: GyroFsr = GyroFsr::DEFAULT;
		const ACCEL_FSR: AccelFsr = AccelFsr::DEFAULT;

		#[inline]
		const fn g(d: i16) -> f32 {
			GYRO_FSR.discrete_to_velocity(d)
		}
		#[inline]
		const fn a(d: i16) -> f32 {
			ACCEL_FSR.discrete_to_accel(d)
		}

		let gyro = vector![g(gyro.x), g(gyro.y), g(gyro.z)];
		let accel = vector![a(accel.x), a(accel.y), a(accel.z)];

		Ok(UnfusedData { accel, gyro })
	}
}

#[allow(dead_code)]
pub fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl DelayMs<u32>,
) -> impl Imu<Data = FusedData> {
	let bmi = Bmi160::new(i2c, delay).expect("Failed to initialize BMI160");
	FusedImu {
		fuser: new_fuser(),
		imu: bmi,
	}
}
