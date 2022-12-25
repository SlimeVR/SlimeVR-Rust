use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
	WindowsX64,
	LinuxX64,
	Unsupported(String),
}

impl Platform {
	pub fn current() -> Platform {
		if cfg!(target_os = "windows") {
			if cfg!(target_arch = "x86_64") {
				Platform::WindowsX64
			} else {
				Platform::Unsupported("windows".to_string())
			}
		} else if cfg!(target_os = "linux") {
			if cfg!(target_arch = "x86_64") {
				Platform::LinuxX64
			} else {
				Platform::Unsupported("linux".to_string())
			}
		} else {
			Platform::Unsupported("unknown".to_string())
		}
	}
}
