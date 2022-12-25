use autoupdater::component::MaybeCrossPlatform;
use color_eyre::{Result, eyre::{ContextCompat, Context}};
use lazy_static::lazy_static;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

lazy_static! {
	static ref SLIME_DIR: MaybeCrossPlatform<PathBuf> =
		MaybeCrossPlatform::Cross(PathBuf::from("todo"));
	static ref STEAM_DIR: MaybeCrossPlatform<PathBuf> =
		MaybeCrossPlatform::Cross(PathBuf::from("todo"));
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum InstallPath {
	/// Just a regular path
	Normal(PathBuf),
	/// A path relative to the SlimeVR installation dir
	RelativeToSlime(PathBuf),
	/// A path relative to the SteamVR installation dir
	RelativeToSteam(PathBuf),
}
impl InstallPath {
	/// Converts to the full absolute path of the file
	pub fn to_path(&self) -> Result<PathBuf> {
		let p = match self {
			InstallPath::Normal(p) => p.to_owned(),
			InstallPath::RelativeToSlime(p) => SLIME_DIR
				.get()
				.wrap_err("No slime install directory for current platform")?
				.join(p),
			InstallPath::RelativeToSteam(p) => STEAM_DIR
				.get()
				.wrap_err("No steam install directory for current platform")?
				.join(p),
		};
		p.absolutize()
			.map(|p| p.to_path_buf())
			.wrap_err("Failed to canonicalize install path")
	}
}
