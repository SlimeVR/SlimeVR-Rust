use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

use super::Platform;

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
    /// Gets the `T` for the current platform.
    pub fn get(&self) -> Option<&T> {
        match self {
            MaybeCrossPlatform::Cross(inner) => Some(inner),
            MaybeCrossPlatform::NotCross(map) => map.get(Platform::current()),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            MaybeCrossPlatform::Cross(inner) => Some(inner),
            MaybeCrossPlatform::NotCross(map) => map.get_mut(Platform::current()),
        }
    }

    pub fn get_owned(self) -> Option<T> {
        match self {
            MaybeCrossPlatform::Cross(inner) => Some(inner),
            MaybeCrossPlatform::NotCross(mut map) => map.remove(Platform::current()),
        }
    }
}
/// Type alias so we don't have long ass names
pub type MCP<T> = MaybeCrossPlatform<T>;
