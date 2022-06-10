mod bone_kind;

use std::process::Output;

pub use self::bone_kind::BoneKind;

use crate::prelude::*;

pub enum BoneEnd {
    Head,
    Tail,
}

pub enum OutputRotation {
    Solved(Global<UnitQuat>),
    Unsolved,
}
impl Default for OutputRotation {
    fn default() -> Self {
        Self::Unsolved
    }
}

/// Bones represent the connections between the [`Joint`]s of the skeelton. Bones have
/// a global rotation, represented as a unit quaternion. To get a bones position, you
/// can get the position of either of its two ends.
///
/// Note that by convention, the directionality of bones points from the skeleton's head
/// downwards. So the head of the bone would connect to the tail of a bone closer to
/// the top of the skeleton. This is simply to give the parent (source in `petgraph)
/// and child (destination) of a bone a consistent meaning.
pub struct Bone {
    kind: BoneKind,
    /// Input rotation in global space. If there is no rotation input for that bone,
    /// it is `None`.
    input_rot_g: Option<Global<UnitQuat>>,
    /// Rotation of the bone with respect to the parent bone at calibration time. Maps
    /// from parent frame to child frame.
    calib_rot_l: Local<UnitQuat>,
    /// Possibly-unsolved rotation of the bone.
    output_rot_g: OutputRotation,
}
impl Bone {
    pub const fn new(kind: BoneKind) -> Self {
        Self {
            kind,
            input_rot_g: None,
            calib_rot_l: kind.calibration_rotation_local(),
            output_rot_g: OutputRotation::Unsolved,
        }
    }

    pub fn input_rotation_mut(&mut self) -> Option<&Global<UnitQuat>> {
        self.input_rot_g.as_mut()
    }

    pub fn output_rotation(&self) -> &OutputRotation {
        &self.output_rot_g
    }
}
