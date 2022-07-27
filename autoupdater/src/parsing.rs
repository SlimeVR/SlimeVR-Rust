//! Describes the format of the versioning file
//!
//! The autoupdater will fetch a versioning file from a github release, which describes
//! what the latest version of all the different pieces of SlimeVR software should be
//! update to, in unison.
//!
//! This file gets deserialized to our [`Components`] datastructure using [`serde`].
//!
//! Note: A lot of the types in this module are generic over `U: TryInto<Url>` and the
//! like. This is to make creating the datatypes using strings instead of `Url`s and
//! `PathBuf`s easier. If you are reading the code and are a rust beginner, and it
//! really confuses you, please post in the SlimeVR discord. I might decide to
//! change/simplify it if it really confuses people.

use derive_more::From;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

/// The location to install a component.
///
/// Single files are placed in this dir, and zip files are unzipped into this dir.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum InstallPath<T = PathBuf> {
    /// Just a regular path
    Normal(T),
    /// A path relative to the SlimeVR installation dir
    RelativeToSlime(T),
    /// A path relative to the SteamVR installation dir
    RelativeToSteam(T),
}
impl<T: Into<PathBuf>> InstallPath<T> {
    /// Converts the inner type to a [`PathBuf`].
    pub fn normalize(self) -> InstallPath<PathBuf> {
        match self {
            Self::Normal(p) => InstallPath::Normal(p.into()),
            Self::RelativeToSlime(p) => InstallPath::RelativeToSlime(p.into()),
            Self::RelativeToSteam(p) => InstallPath::RelativeToSteam(p.into()),
        }
    }
}

/// This enum allows us to represent a `T` that may or may not depend on the platform
/// that we wish to install for.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, From)]
#[serde(untagged)]
pub enum MaybeCrossPlatform<T> {
    /// This `T` is the same across all platforms.
    Cross(T),
    /// This `T` depends on the `Platform`
    NotCross(HashMap<Platform, T>),
}
impl<T> MaybeCrossPlatform<T> {
    /// Attempts to map from `T` to `U` using the function provided. If any of
    /// the elements fail with an `Err(E)`, the function will stop and return the error.
    pub fn try_map<U>(
        self,
        mut f: impl FnMut(T) -> Result<U>,
    ) -> Result<MaybeCrossPlatform<U>> {
        Ok(match self {
            Self::Cross(t) => MaybeCrossPlatform::Cross(f(t)?),
            Self::NotCross(m) => MaybeCrossPlatform::NotCross(
                m.into_iter()
                    .map(|(key, val)| f(val).map(|val| (key, val)))
                    .collect::<Result<_>>()?,
            ),
        })
    }

    /// Maps the `T` to `U` using the function provided.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> MaybeCrossPlatform<U> {
        match self {
            Self::Cross(t) => MaybeCrossPlatform::Cross(f(t)),
            Self::NotCross(m) => MaybeCrossPlatform::NotCross(
                m.into_iter().map(|(key, value)| (key, f(value))).collect(),
            ),
        }
    }
}
/// Type alias so we don't have long ass names
type MCP<T> = MaybeCrossPlatform<T>;

/// Represents a target platform for SlimeVR
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Windows64,
    Linux64,
    /// A platform we don't understand.
    ///
    /// We might get this variant if we added a new platform that we want to support,
    /// but the updater hasn't updated to understand it yet.
    #[serde(other)]
    Unknown,
}

/// Describes all the information about a component and how to install it.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ComponentInfo<U = Url, P = PathBuf> {
    /// The URL from which this component is downloaded.
    download_url: MCP<U>,
    /// The dir to which this component is installed.
    install_dir: MCP<InstallPath<P>>,
}
impl<U: TryInto<Url, Error = url::ParseError>, P: Into<PathBuf>> ComponentInfo<U, P> {
    /// Converts the inner types into the default type arguments.
    pub fn normalize(self) -> Result<ComponentInfo> {
        Ok(ComponentInfo {
            download_url: self
                .download_url
                .try_map(|url| url.try_into().wrap_err("Failed to parse"))?,
            install_dir: self.install_dir.map(InstallPath::normalize),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentName {
    Overlay,
    Server,
    Feeder,
    Driver,
    Gui,
    AutoUpdater,
    /// The name of a component we don't understand.
    ///
    /// This might happen if the autoupdater has not yet updated to know about this
    /// component.
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Components(HashMap<ComponentName, ComponentInfo>);

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref EXAMPLE_STRUCT: Components = Components(HashMap::from([
            (
                ComponentName::Server,
                ComponentInfo {
                    download_url: "https://github.com/SlimeVR/SlimeVR-Server/releases/download/v0.2.0/slimevr.jar".into(),
                    install_dir: InstallPath::RelativeToSlime("").into(),
                }.normalize().unwrap()
            ),
            (
                ComponentName::Overlay,
                ComponentInfo {
                    download_url: MCP::NotCross(HashMap::from([(
                        Platform::Windows64,
                        "https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/windows-x64.zip",
                    ),
                    (
                        Platform::Linux64,
                        "https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/linux-x64.zip"
                    )])),
                    install_dir: InstallPath::RelativeToSlime("overlay").into()
                }.normalize().unwrap()
            ),
            (
                ComponentName::Unknown,
                ComponentInfo {
                    download_url: "https://github.com/SlimeVR/whatever".into(),
                    install_dir: InstallPath::Normal(PathBuf::from(r"D:\s\nuts")).into()
                }.normalize().unwrap()
            ),
        ]));
    }

    const EXAMPLE_STR: &str = r#"
        server:
            download_url: https://github.com/SlimeVR/SlimeVR-Server/releases/download/v0.2.0/slimevr.jar
            install_dir:
                relative_to_slime: ""
        overlay:
            download_url:
                windows64: https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/windows-x64.zip
                linux64: https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/linux-x64.zip
            install_dir:
                relative_to_slime: overlay
        a_new_component:
            download_url: https://github.com/SlimeVR/whatever
            install_dir:
                normal: D:\s\nuts
        "#;

    #[test]
    fn test_round_trip() -> eyre::Result<()> {
        let deserialized: Components = serde_yaml::from_str(EXAMPLE_STR)?;
        let round_tripped: Components =
            serde_yaml::from_str(&serde_yaml::to_string(&deserialized)?)?;

        assert_eq!(deserialized, round_tripped);
        println!("Example:\n{:#?}", *EXAMPLE_STRUCT);
        println!("Deserialized:\n{:#?}", deserialized);
        assert_eq!(deserialized, *EXAMPLE_STRUCT);
        Ok(())
    }
}
