use vqf::{Vqf, VqfParameters};

use crate::imu::fusion::Fuser;
use crate::imu::{FusedData, UnfusedData};

/// Extended Kalman filtering in direction cosine matrix formation
pub struct VqfFusion {
	vqf: Vqf,
}

impl VqfFusion {
	#[allow(dead_code)]
	pub fn new() -> Self {
		let param = VqfParameters::default();
		Self {
			vqf: Vqf::new(0.01818, 0.01818, 0.01818, param),
		}
	}
}

impl Fuser for VqfFusion {
	fn process(&mut self, unfused: &UnfusedData) -> FusedData {
		//elapsed
		self.vqf
			.update(unfused.gyro.clone(), unfused.accel.clone(), None);

		let q = self.vqf.getQuat6D();
		FusedData { q }
	}
}
