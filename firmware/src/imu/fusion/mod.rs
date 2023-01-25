use firmware_protocol::ImuType;

mod naive;

use super::Quat;

pub use naive::Naive;

pub trait FusedImu {
	type Error: core::fmt::Debug;

	const IMU_TYPE: ImuType;
	async fn quat(&mut self) -> Result<Quat, Self::Error>;
}
