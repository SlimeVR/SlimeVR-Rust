use std::{collections::HashMap, fmt::Display, fs::File, io, path::PathBuf};

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{
	platform::Platform,
	util::{Selectable, SelectableHashMap},
};

use super::{
	incompatible::{IncompatibilityReason, IncompatibleComponent},
	MaybeCrossPlatform,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableComponentsFile {
	components: HashMap<String, Component>,
}

impl AvailableComponentsFile {
	pub fn load(path: PathBuf) -> io::Result<AvailableComponentsFile> {
		let file = File::open(path)?;
		match serde_yaml::from_reader(file) {
			Ok(components) => Ok(components),
			Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
	display_name: String,
	version: Version,
	platforms: Vec<Platform>,
	dependencies: Option<MaybeCrossPlatform<Vec<String>>>,
}

impl Component {
	pub fn incompatible_because(
		&self,
		reason: IncompatibilityReason,
	) -> IncompatibleComponent {
		IncompatibleComponent::new(
			self.display_name.clone(),
			reason,
			self.version.clone(),
			self.platforms.clone(),
		)
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

impl Selectable<String> for Component {
	fn dependencies(&self) -> Option<&[String]> {
		match &self.dependencies {
			Some(d) => d.get().map(|v| v.as_slice()),
			None => None,
		}
	}
}

impl Display for Component {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} v{}", self.display_name, self.version)
	}
}

pub struct Components {
	compatible: HashMap<String, Component>,
	incompatible: HashMap<String, IncompatibleComponent>,
}
impl Components {
	pub fn compatible(&self) -> &HashMap<String, Component> {
		&self.compatible
	}

	pub fn incompatible(&self) -> &HashMap<String, IncompatibleComponent> {
		&self.incompatible
	}

	/// Converts the `Components` into a `SelectableHashMap` of all compatible `Component`s.
	pub fn into_selectable_hashmap(self) -> SelectableHashMap<String, Component> {
		SelectableHashMap::new(self.compatible)
	}
}

impl From<AvailableComponentsFile> for Components {
	fn from(available: AvailableComponentsFile) -> Self {
		let platform = Platform::current();

		let mut compatible = HashMap::new();
		let mut incompatible = HashMap::new();

		for (name, component) in available.components {
			if component.platforms.contains(&platform) {
				compatible.insert(name, component);
			} else {
				incompatible.insert(
					name,
					component.incompatible_because(
						IncompatibilityReason::PlatformNotSupported(platform.clone()),
					),
				);
			}
		}

		Components {
			compatible,
			incompatible,
		}
	}
}
