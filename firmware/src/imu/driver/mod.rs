mod bmi160;
mod ism330dhcx;
mod mpu6050;

use firmware_protocol::ImuType;

use super::Vec3;

pub use self::bmi160::Bmi160;
pub use self::ism330dhcx::Ism330Dhcx;
pub use self::mpu6050::Mpu6050;

#[derive(Default, Debug)]
pub struct ImuData {
	/// Acceleration vector in m/s^2
	pub accel: Vec3,
	/// Gyroscope angular rate in rad/s
	pub gyro: Vec3,
	/// Vector aligned with the earths magnetic field, in arbitrary units
	pub mag: Option<Vec3>,
}

pub trait Imu {
	type Error: core::fmt::Debug;

	const IMU_TYPE: ImuType;
	async fn data(&mut self) -> Result<ImuData, Self::Error>;
}
