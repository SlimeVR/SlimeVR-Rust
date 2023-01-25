use firmware_protocol::ImuType;

use crate::imu::{Imu, Quat};

use super::FusedImu;

/// Simply applies a constant adjustment to the sensor data
pub struct Rotate<T> {
	inner: T,
	quat: Quat,
}

impl<T> Rotate<T> {
	pub fn new(inner: T, quat: Quat) -> Rotate<T> {
		Rotate { inner, quat }
	}
}

impl<T: FusedImu> FusedImu for Rotate<T> {
	type Error = T::Error;

	const IMU_TYPE: ImuType = T::IMU_TYPE;

	async fn quat(&mut self) -> Result<Quat, Self::Error> {
		Ok(self.inner.quat().await? * self.quat)
	}
}
