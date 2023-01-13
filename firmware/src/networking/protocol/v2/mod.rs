//! This will become the new protocol task. However we are not finished developing it
//! yet so instead of working in a long-lived feature branch, I am merging it to main
//! so that we can all contribute code to it and later turn it on.

mod imu_handle;
mod network_handle;
mod serialize;
pub use self::imu_handle::ImuHandle;
pub use self::network_handle::NetworkHandle;
pub use self::serialize::{Serialize, SerializeExact};

// TODO: Start writing the protocol task, we will see quite quickly what this needs.
pub struct Protocol {}
impl Protocol {
	pub fn new() -> (Self, ImuHandle, NetworkHandle) {
		todo!()
	}
}
