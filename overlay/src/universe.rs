use ovr_overlay as ovr;
use serde::{
	de::{self, Deserializer, Visitor},
	Deserialize,
};
use serde_json::from_slice;
use std::fmt;

// From https://github.com/serde-rs/json/issues/412#issuecomment-365856864
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct UniverseId(u64);

impl<'de> Deserialize<'de> for UniverseId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct IdVisitor;

		impl<'de> Visitor<'de> for IdVisitor {
			type Value = UniverseId;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("universe ID as a number or string")
			}

			fn visit_u64<E>(self, id: u64) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(UniverseId(id))
			}

			fn visit_str<E>(self, id: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				id.parse().map(UniverseId).map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_any(IdVisitor)
	}
}

#[derive(Debug, Deserialize)]
pub(crate) struct UniverseTranslation {
	pub translation: nalgebra::Translation3<f32>,
	pub yaw: f32,
}

#[derive(Debug, Deserialize)]
struct Universe {
	#[serde(rename = "universeID")]
	universe_id: UniverseId,
	standing: UniverseTranslation,
}

#[derive(Debug, Deserialize)]
struct Document {
	universes: Vec<Universe>,
}

pub(crate) fn search_universe(
	context: &ovr::Context,
	universe_id: u64,
) -> Option<UniverseTranslation> {
	let chaperone_setup = context.chaperone_setup_mngr().export_live_to_buffer()?;

	// note: in theory we could parse in a streaming fashion using an appropriate json library
	//  instead of reading in everything all at once and then filtering.
	//  in practice this will probably never matter.
	let document: Document = match from_slice(chaperone_setup.as_bytes()) {
		Ok(res) => res,
		Err(err) => {
			// TODO: only log this once?
			log::error!("Error parsing universe json: {}", err);
			return None;
		}
	};

	document
		.universes
		.into_iter()
		.find(|u| u.universe_id.0 == universe_id)
		.map(|u| u.standing)
}
