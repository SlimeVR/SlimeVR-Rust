mod dcm;
mod stubbed;
mod vqf;

pub use self::dcm::Dcm;
pub use self::stubbed::Stubbed;
pub use self::vqf::VqfFusion;

use crate::imu::{FusedData, Imu, UnfusedData};

use firmware_protocol::ImuType;

/// Represents a sensor fusion algorithm that will take an imu's `UnfusedData` and
/// do math to turn it into `FusedData`, suitable for use as orientation.
pub trait Fuser {
	// Note: Intentionally not async, this should only be doing math, not io or
	// any internal awaiting.
	fn process(&mut self, unfused: &UnfusedData) -> FusedData;
}

/// Combines an unfused `Imu` with a `Fuser`.
pub struct FusedImu<I: Imu, F: Fuser> {
	pub imu: I,
	pub fuser: F,
}
impl<I: Imu<Data = UnfusedData>, F: Fuser> Imu for FusedImu<I, F> {
	type Error = I::Error;
	type Data = FusedData;

	const IMU_TYPE: ImuType = I::IMU_TYPE;

	async fn next_data(&mut self) -> Result<Self::Data, Self::Error> {
		let unfused = self.imu.next_data().await?;
		Ok(self.fuser.process(&unfused))
	}
}

/// Builds a new fuser. The concrete impl is determined by a feature flag.
pub fn new_fuser() -> impl Fuser {
	#[cfg(feature = "fusion-stubbed")]
	let fusion_algorithm = Stubbed::new();
	#[cfg(feature = "fusion-dcm")]
	let fusion_algorithm = Dcm::new();
	#[cfg(feature = "fusion-vqf")]
	let fusion_algorithm = VqfFusion::new();

	fusion_algorithm
}
