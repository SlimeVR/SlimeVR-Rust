//! Describes skeletal bones and their relationships.
//!
//! For more info about skeletal bones and the role they play in the skeleton,
//! see the module-level documentation on [`crate::skeleton`].

mod bone_kind;
pub mod bone_map;

pub use self::bone_kind::BoneKind;
#[doc(inline)]
pub use self::bone_map::BoneMap;
