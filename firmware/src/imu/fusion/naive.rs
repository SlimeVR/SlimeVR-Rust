use firmware_protocol::ImuType;

use crate::imu::{Imu, Quat};

use super::FusedImu;

/// Minimalist integrating sensor "fusion"
pub struct Naive<T> {
	inner: T,
	quat: Quat,
}

impl<T> Naive<T> {
	pub fn new(inner: T) -> Naive<T> {
		Naive {
			inner,
			quat: Quat::identity(),
		}
	}
}

impl<T: Imu> FusedImu for Naive<T> {
	type Error = T::Error;

	const IMU_TYPE: ImuType = T::IMU_TYPE;

	async fn quat(&mut self) -> Result<Quat, Self::Error> {
		// Load rotation quaternion from underlying driver
		let data = self.inner.data().await?;
		let rot = Quat::from_euler_angles(data.gyro.x, data.gyro.y, data.gyro.z);

		// Apply newest movement
		self.quat *= rot;
		Ok(self.quat)
	}
}
