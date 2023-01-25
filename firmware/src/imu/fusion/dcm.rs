use dcmimu::DCMIMU;
use embassy_time::Instant;
use firmware_protocol::ImuType;

use crate::imu::{Imu, ImuData, Quat};

use super::FusedImu;

/// Extended Kalman filtering in direction cosine matrix formation
pub struct Dcm<T> {
	inner: T,
	dcm: DCMIMU,
	last: Instant,
}

impl<T> Dcm<T> {
	pub fn new(inner: T) -> Self {
		Self {
			inner,
			dcm: DCMIMU::new(),
			last: Instant::now(),
		}
	}
}

impl<T: Imu> FusedImu for Dcm<T> {
	type Error = T::Error;

	const IMU_TYPE: ImuType = T::IMU_TYPE;

	async fn quat(&mut self) -> Result<crate::imu::Quat, Self::Error> {
		let ImuData { accel, gyro, .. } = self.inner.data().await?;

		let elapsed = self.last.elapsed();
		self.last += elapsed;

		let (euler, _) = self.dcm.update(
			(gyro.x, gyro.y, gyro.z),
			(accel.x, accel.y, accel.z),
			elapsed.as_micros() as f32 / 1_000_000.0,
		);

		Ok(Quat::from_euler_angles(euler.roll, euler.pitch, euler.roll))
	}
}
