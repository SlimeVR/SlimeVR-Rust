use ism330dhcx::{ctrl2g::Odr, Ism330Dhcx as Driver};

use embassy_time::{Duration, Ticker};
use firmware_protocol::ImuType;
use futures_util::StreamExt;

use crate::{aliases::I2c, imu::Vec3};

use super::{Imu, ImuData};

pub struct Ism330Dhcx<I> {
	driver: Driver,
	i2c: I,
	ticker: Ticker,
}

impl<I: I2c> Ism330Dhcx<I> {
	pub fn new(mut i2c: I) -> Result<Self, <I as I2c>::Error> {
		let mut driver = Driver::new(&mut i2c)?;

		// Set data rate to something reasonable
		let odr = Odr::Hz104;
		driver.ctrl2g.set_gyroscope_data_rate(&mut i2c, odr)?;

		// Sample gyro only when needed
		let gyro_rate =
			Duration::from_secs(1) / driver.ctrl2g.gyroscope_data_rate() as u32;
		let ticker = Ticker::every(gyro_rate);

		Ok(Self {
			driver,
			i2c,
			ticker,
		})
	}
}

impl<I: I2c> Imu for Ism330Dhcx<I> {
	type Error = <I as I2c>::Error;

	const IMU_TYPE: ImuType = ImuType::Unknown(0xFF);

	async fn data(&mut self) -> Result<ImuData, Self::Error> {
		// await for new data
		self.ticker.next().await;

		// Gs
		let accel = self.driver.get_accelerometer(&mut self.i2c)?;
		// rad/s
		let gyro = self.driver.get_gyroscope(&mut self.i2c)?;

		Ok(ImuData {
			accel: Vec3::new(accel[0] as f32, accel[1] as f32, accel[2] as f32) * 9.81,
			gyro: Vec3::new(gyro[0] as f32, gyro[1] as f32, gyro[2] as f32),
			mag: None,
		})
	}
}
