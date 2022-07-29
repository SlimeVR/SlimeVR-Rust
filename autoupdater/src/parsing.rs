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
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

/// The location to install a component.
///
/// Single files are placed in this dir, and zip files are unzipped into this dir.
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
pub struct ComponentInfo {
    /// The URL from which this component is downloaded.
    download_url: MCP<Url>,
    /// The dir to which this component is installed.
    install_dir: MCP<InstallPath>,
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

    // These have been split out because rustfmt shits the bed when lines exceed the
    // max_width and can't be wrapped for it
    const SERVER_WINDOWS: &str = "https://github.com/SlimeVR/SlimeVR-Server/releases/download/v0.2.0/slimevr.jar";
    const OVERLAY_WINDOWS: &str = "https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/windows-x64.zip";
    const OVERLAY_LINUX: &str = "https://github.com/SlimeVR/SlimeVR-Rust/releases/download/overlay-latest/linux-x64.zip";

    lazy_static! {
        static ref EXAMPLE_STRUCT: Components = {
            let mut components = HashMap::new();
            components.insert(
                ComponentName::Server,
                ComponentInfo {
                    download_url: Url::parse(SERVER_WINDOWS).unwrap().into(),
                    install_dir: InstallPath::RelativeToSlime(PathBuf::from("")).into(),
                },
            );
            components.insert(
                ComponentName::Overlay,
                ComponentInfo {
                    download_url: MCP::NotCross(HashMap::from([
                        (Platform::Windows64, Url::parse(OVERLAY_WINDOWS).unwrap()),
                        (Platform::Linux64, Url::parse(OVERLAY_LINUX).unwrap()),
                    ])),
                    install_dir: InstallPath::RelativeToSlime(PathBuf::from("overlay"))
                        .into(),
                },
            );
            components.insert(
                ComponentName::Unknown,
                ComponentInfo {
                    download_url: Url::parse("https://github.com/SlimeVR/whatever")
                        .unwrap()
                        .into(),
                    install_dir: InstallPath::Normal(PathBuf::from(r"D:\s\nuts"))
                        .into(),
                },
            );
            Components(components)
        };
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
