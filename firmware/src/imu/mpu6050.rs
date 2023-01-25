use super::{Imu, Quat};
use crate::aliases::I2c;
use crate::utils;

use defmt::{debug, trace, warn};
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::error::InitError;
use mpu6050_dmp::sensor::Mpu6050 as LibMpu;

pub struct Mpu6050<I: I2c> {
	mpu: LibMpu<I>,
	fifo_buf: [u8; 28],
}
impl<I: I2c> Mpu6050<I> {
	pub fn new(i2c: I, delay: &mut impl DelayMs<u32>) -> Result<Self, InitError<I>> {
		debug!("Constructing MPU...");
		let addr = Address::from(0x68);
		debug!("I2C address: {:x}", addr.0);

		utils::retry(
			4,
			i2c,
			|mut i2c| {
				delay.delay_ms(100);
				trace!("Flushing I2C with bogus data");
				let _ = i2c.write(addr.0, &[0]);
				delay.delay_ms(100);
				trace!("Constructing IMU");
				let mut mpu = LibMpu::new(i2c, addr)
					// Map converts from struct -> tuple
					.map_err(|InitError { i2c, error }| (i2c, error))?;
				debug!("Constructed MPU");
				delay.delay_ms(100);
				if let Err(error) = mpu.initialize_dmp(delay) {
					return Err((mpu.release(), error));
				}
				debug!("Initialized DMP");
				Ok(Self {
					mpu,
					fifo_buf: [0; 28],
				})
			},
			|i| warn!("Retrying IMU connection (attempts so far: {})", i + 1),
		)
		// Map converts from tuple -> struct
		.map_err(|(i2c, error)| InitError { i2c, error })
	}
}

impl<I: I2c> Imu for Mpu6050<I> {
	type Error = mpu6050_dmp::error::Error<I>;

	const IMU_TYPE: ImuType = ImuType::Mpu6050;

	fn quat(&mut self) -> nb::Result<Quat, Self::Error> {
		if self.mpu.get_fifo_count()? >= 28 {
			let data = self.mpu.read_fifo(&mut self.fifo_buf)?;
			let opt = data.get(..16);
			if let Some(data) = opt {
				let q = mpu6050_dmp::quaternion::Quaternion::from_bytes(data).unwrap();
				let q = nalgebra::Quaternion {
					coords: nalgebra::vector![q.x, q.y, q.z, q.w],
				};
				Ok(Quat::from_quaternion(q))
			} else {
				Err(nb::Error::WouldBlock)
			}
		} else {
			Err(nb::Error::WouldBlock)
		}
	}
}

pub fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl DelayMs<u32>,
) -> impl crate::imu::Imu {
	Mpu6050::new(i2c, delay).expect("Failed to initialize MPU6050")
}
