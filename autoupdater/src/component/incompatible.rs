use std::fmt::Display;

use semver::Version;

use crate::platform::Platform;

#[derive(Debug, Clone)]
pub enum IncompatibilityReason {
	PlatformNotSupported(Platform),
}

#[derive(Debug, Clone)]
pub struct IncompatibleComponent {
	display_name: String,
	reason: IncompatibilityReason,
	version: Version,
	platforms: Vec<Platform>,
}

impl IncompatibleComponent {
	pub(in super) fn new(
		display_name: String,
		reason: IncompatibilityReason,
		version: Version,
		platforms: Vec<Platform>,
	) -> Self {
		Self {
			display_name,
			reason,
			version,
			platforms,
		}
	}

	pub fn reason(&self) -> &IncompatibilityReason {
		&self.reason
	}

	pub fn version(&self) -> &Version {
		&self.version
	}

	pub fn display_name(&self) -> &str {
		&self.display_name
	}

	pub fn platforms(&self) -> &Vec<Platform> {
		&self.platforms
	}
}

impl Display for IncompatibleComponent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} v{} because {:?}",
			self.display_name, self.version, self.reason
		)
	}
}
