use dcmimu::DCMIMU;
use embassy_time::Instant;

use crate::imu::fusion::Fuser;
use crate::imu::{FusedData, Quat, UnfusedData};

/// Extended Kalman filtering in direction cosine matrix formation
pub struct Dcm {
	dcm: DCMIMU,
	last: Instant,
}

impl Dcm {
	#[allow(dead_code)]
	pub fn new() -> Self {
		Self {
			dcm: DCMIMU::new(),
			last: Instant::now(),
		}
	}
}

impl Fuser for Dcm {
	fn process(&mut self, unfused: &UnfusedData) -> FusedData {
		let last = self.last;
		self.last = Instant::now();
		let elapsed = self.last - last;
		let elapsed = elapsed.as_micros() as f32 / 1_000_000.0;

		let UnfusedData { accel, gyro } = unfused;

		// TODO: Check that these euler angle convention matches
		let (euler, _) = self.dcm.update(
			(gyro.x, gyro.y, gyro.z),
			(accel.x, accel.y, accel.z),
			elapsed,
		);

		let q = Quat::from_euler_angles(euler.roll, euler.pitch, euler.yaw);
		FusedData { q }
	}
}
