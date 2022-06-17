//! Contains skeletal bones and their relationships.
//!
//! For more info about skeletal bones and the role it plays in the skeleton,
//! see the module-level documentation on [`crate::skeleton`].

mod bone_kind;
mod bone_map;

pub use self::bone_kind::BoneKind;
pub use self::bone_map::BoneMap;
