mod bone_kind;

pub use self::bone_kind::BoneKind;

use crate::UnitQuat;

pub enum BoneEnd {
    Head,
    Tail,
}

/// Bones represent the connections between the [`Joint`]s of the skeelton. Bones have
/// a global rotation, represented as a unit quaternion. To get a bones position, you
/// can get the position of either of its two ends.
///
/// Note that by convention, the directionality of bones points from the skeleton's head
/// downwards. So the head of the bone would be at the upper chest and  This is simply to give the head (destination in petgraph) and tail (destination) of a
/// bone a consistent meaning.
pub struct Bone {
    kind: BoneKind,
    global_rot: UnitQuat,
}
impl Bone {
    pub fn new(kind: BoneKind) -> Self {
        Self {
            kind,
            global_rot: UnitQuat::default(),
        }
    }
}
