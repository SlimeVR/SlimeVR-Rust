mod bmi160;
mod lsm6ds33;
mod mpu6050;

use firmware_protocol::ImuType;

use super::Vec3;

pub use self::bmi160::Bmi160;
pub use self::lsm6ds33::Lsm6Ds33;
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
