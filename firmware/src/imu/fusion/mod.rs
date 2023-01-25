use firmware_protocol::ImuType;

mod dcm;
mod naive;

use super::Quat;

pub use dcm::Dcm;
pub use naive::Naive;

pub trait FusedImu {
	type Error: core::fmt::Debug;

	const IMU_TYPE: ImuType;
	async fn quat(&mut self) -> Result<Quat, Self::Error>;
}
