use std::{fs::File, path::PathBuf, io, collections::HashMap};

use semver::Version;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledComponentsFile {
	components: HashMap<String, InstalledComponent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledComponent {
	version: Version,
}

impl InstalledComponentsFile {
	pub fn load(path: PathBuf) -> io::Result<InstalledComponentsFile> {
		match path.exists() {
			true => {
				let file = File::open(path)?;
				match serde_yaml::from_reader(file) {
					Ok(components) => Ok(components),
					Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
				}
			}
			false => Ok(InstalledComponentsFile {
				components: HashMap::new(),
			}),
		}
	}

	pub fn save(&self, path: PathBuf) -> io::Result<()> {
		let file = File::create(path)?;
		match serde_yaml::to_writer(file, self) {
			Ok(_) => Ok(()),
			Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
		}
	}
}
