use core::f32::consts::PI;

use embassy_time::Instant;

use crate::imu::{FusedData, Fuser, Quat, UnfusedData};

/// A fake fuser that just rotates around the x axis.
pub struct Stubbed(Instant);
impl Stubbed {
	pub fn new() -> Self {
		Self(Instant::now())
	}
}
impl Fuser for Stubbed {
	fn process(&mut self, _unfused: &UnfusedData) -> FusedData {
		let dt = self.0.elapsed();
		// Convert to seconds as float
		let dt = dt.as_micros() as f32 / 1_000_000.;

		const ROT_RATE: f32 = PI / 2.; // 90 degrees per second
		FusedData {
			q: Quat::from_axis_angle(&nalgebra::Vector3::x_axis(), dt * ROT_RATE),
		}
	}
}
