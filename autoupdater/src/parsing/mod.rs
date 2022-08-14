//! Describes the format of the versioning file
//!
//! The versioning file describes what the latest version of all the different pieces
//! of SlimeVR software should be updated to, so that the update can be performed
//! atomically.
//!
//! This file gets deserialized to our [`Components`] datastructure using [`serde`].

mod install_path;
mod mcp;
pub use install_path::InstallPath;
pub use mcp::{MaybeCrossPlatform, MCP};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

// These give us the ability to use #[serde(other)] on string tuples
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

/// Represents a target platform for SlimeVR
#[derive(
    Deserialize_enum_str, Serialize_enum_str, Clone, Debug, Eq, PartialEq, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Windows64,
    Linux64,
    /// A platform we don't understand.
    ///
    /// We might get this variant if we added a new platform that we want to support,
    /// but the updater hasn't updated to understand it yet.
    #[serde(other)]
    Unknown(String),
}
impl Platform {
    pub fn current() -> &'static Platform {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return &Platform::Windows64;
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return &Platform::Linux64;
        lazy_static! {
            static ref PLATFORM: Platform = Platform::Unknown("unknown".to_string());
        }
        #[allow(unused)]
        &PLATFORM
    }
}

/// Describes all the information about a component and how to install it.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ComponentInfo {
    /// The URL from which this component is downloaded.
    pub download_url: MCP<Url>,
    /// The dir to which this component is installed.
    pub install_dir: MCP<InstallPath>,
}

#[derive(Deserialize_enum_str, Serialize_enum_str, Debug, Hash, Eq, PartialEq)]
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
    Unknown(String),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Components(pub HashMap<ComponentName, ComponentInfo>);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
                ComponentName::Unknown("a_new_component".to_string()),
                ComponentInfo {
                    download_url: Url::parse("https://github.com/SlimeVR/whatever")
                        .unwrap()
                        .into(),
                    install_dir: InstallPath::Normal(PathBuf::from(r"D:\s\nuts"))
                        .into(),
                },
            );
            components.insert(
                ComponentName::Unknown("another_component".to_string()),
                ComponentInfo {
                    download_url: Url::parse("https://github.com/slimeVR/another")
                        .unwrap()
                        .into(),
                    install_dir: MCP::NotCross(HashMap::from([
                        (Platform::Windows64, InstallPath::Normal("".into())),
                        (Platform::Linux64, InstallPath::Normal("".into())),
                    ])),
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
        another_component:
            download_url: https://github.com/slimeVR/another
            install_dir:
                windows64:
                    normal: ""
                linux64:
                    normal: ""
    "#;

    #[test]
    fn test_round_trip() -> color_eyre::Result<()> {
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
