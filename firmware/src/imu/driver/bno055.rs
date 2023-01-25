use bno055::{BNO055OperationMode as Mode, Bno055 as Driver, Error};
use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;

use crate::{
	aliases::I2c,
	imu::{FusedImu, Quat, Vec3},
};

use super::{Imu, ImuData};

pub struct Bno055<I, const FUSED: bool = true> {
	driver: Driver<I>,
}

impl<I: I2c, const F: bool> Bno055<I, F> {
	pub fn new(
		i2c: I,
		delay: &mut impl DelayMs<u16>,
	) -> Result<Self, Error<<I as I2c>::Error>> {
		let mut driver = Driver::new(i2c);

		driver.init(delay)?;

		if F {
			// Use internal fusion using gyro and accel, no mag
			driver.set_mode(Mode::IMU, delay)?;
		} else {
			// Do not perform fusion on-chip
			driver.set_mode(Mode::AMG, delay)?;
		}

		Ok(Self { driver })
	}
}

// Using on-chip fusion
impl<I: I2c> FusedImu for Bno055<I, true> {
	type Error = Error<<I as I2c>::Error>;

	const IMU_TYPE: ImuType = ImuType::Bno055;

	async fn quat(&mut self) -> Result<Quat, Self::Error> {
		let quat = self.driver.quaternion()?;

		// why can't we access the timing config... guh
		yield_now().await;

		Ok(Quat::from_quaternion(quat.into()))
	}
}

// We like our own fusion code
impl<I: I2c> Imu for Bno055<I, false> {
	type Error = Error<<I as I2c>::Error>;

	const IMU_TYPE: ImuType = ImuType::Bno055;

	async fn data(&mut self) -> Result<ImuData, Self::Error> {
		let accel: Vec3 = self.driver.accel_data()?.into();
		let gyro: Vec3 = self.driver.gyro_data()?.into();
		let mag: Vec3 = self.driver.mag_data()?.into();

		// why can't we access the timing config... guh
		yield_now().await;

		Ok(ImuData {
			accel,
			gyro: gyro * 1f32.to_radians(),
			mag: Some(mag),
		})
	}
}
