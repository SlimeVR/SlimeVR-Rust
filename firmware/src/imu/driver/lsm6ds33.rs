use crate::aliases::I2c;

use embassy_time::{Duration, Ticker};
use firmware_protocol::ImuType;
use futures_util::StreamExt;
use lsm6ds33::Lsm6ds33 as Driver;

use super::{Imu, ImuData, Vec3};

pub struct Lsm6Ds33<I> {
	driver: Driver<I>,
	ticker: Ticker,
}
impl<I: I2c> Lsm6Ds33<I> {
	pub fn new(i2c: I) -> Result<Self, lsm6ds33::Error<<I as I2c>::Error>> {
		let mut driver = Driver::new(i2c, 0x6A).map_err(|(_, e)| e)?;

		driver.set_accelerometer_output(lsm6ds33::AccelerometerOutput::Rate104)?;
		driver.set_gyroscope_output(lsm6ds33::GyroscopeOutput::Rate104)?;

		let ticker = Ticker::every(Duration::from_secs(1) / 104);

		Ok(Lsm6Ds33 { driver, ticker })
	}
}

impl<I: I2c> Imu for Lsm6Ds33<I> {
	type Error = lsm6ds33::Error<<I as I2c>::Error>;

	const IMU_TYPE: ImuType = ImuType::Unknown(0xFF);

	async fn data(&mut self) -> Result<ImuData, Self::Error> {
		self.ticker.next().await;

		let (ax, ay, az) = self.driver.read_accelerometer()?;
		let (gx, gy, gz) = self.driver.read_gyro()?;

		Ok(ImuData {
			accel: Vec3::new(ax, ay, az),
			gyro: Vec3::new(gx, gy, gz),
			mag: None,
		})
	}
}
