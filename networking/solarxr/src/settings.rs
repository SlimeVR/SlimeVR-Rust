use solarxr_protocol::pub_sub::KeyValues;

#[derive(Debug)]
pub struct DisplaySettings {
	pub is_visible: bool,
	pub is_mirrored: bool,
}
impl DisplaySettings {
	pub const IS_VISIBLE: &str = "is_visible";
	pub const IS_MIRRORED: &str = "is_mirrored";

	/// Builds `DisplaySettings` from a flatbuffer
	pub fn from_fb(kv: KeyValues<'_>) -> Option<Self> {
		let (Some(keys), Some(values)) = (kv.keys(), kv.values()) else {
			return None;
		};

		if keys.len() != values.len() {
			log::warn!("Keys and values were not same length!");
			return None;
		}

		// If any settings are unconfigured, we will use the defaults.
		let mut result = Self::default();
		for i in 0..keys.len() {
			let k = keys.get(i);
			let v = values.get(i);

			match k {
				Self::IS_VISIBLE => {
					result.is_visible = v == "true";
				}
				Self::IS_MIRRORED => {
					result.is_mirrored = v == "true";
				}
				_ => (), // Ignore unexpected keys - publisher may be on a newer schema
			}
		}

		Some(result)
	}
}
#[allow(clippy::derivable_impls)]
impl Default for DisplaySettings {
	fn default() -> Self {
		Self {
			is_visible: false,
			is_mirrored: false,
		}
	}
}
