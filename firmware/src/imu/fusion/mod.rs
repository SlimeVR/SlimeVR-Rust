mod dcm;
mod stubbed;

pub use self::dcm::Dcm;
pub use self::stubbed::Stubbed;

use super::Fuser;

/// Builds a new fuser. The concrete impl is determined by a feature flag.
pub fn new_fuser() -> impl Fuser {
	#[cfg(feature = "fusion-stubbed")]
	let f = Stubbed::new();
	#[cfg(feature = "fusion-dcm")]
	let f = Dcm::new();

	f
}
