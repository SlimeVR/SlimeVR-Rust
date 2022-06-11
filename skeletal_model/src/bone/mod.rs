mod bone_kind;
mod bone_map;

pub use self::bone_kind::BoneKind;
pub use self::bone_map::BoneMap;

use crate::prelude::*;

/// Bones represent the connections between the [`Joint`]s of the skeleton. Bones have
/// a global rotation, represented as a unit quaternion. To get a bones position, you
/// can get the position of either of its two attached `Joint`s.
///
/// Note that by convention, the directionality of bones points towards the top of the
/// skeleton. So the head of the bone would connect to the tail of a bone closer to
/// the top of the skeleton. This is simply to give the parent and child of a bone a
/// consistent meaning.
pub struct Bone {
    kind: BoneKind,
    /// Input rotation in global space. If it is unconstrained, it is `None`.
    input_rot_g: Option<Global<UnitQuat>>,
    /// Rotation of the bone with respect to the parent bone at calibration time. Maps
    /// from parent frame to child frame.
    calib_rot_l: Local<UnitQuat>,
    /// Length of the bone. May be set by the user, or may be computed at calibration.
    length: f32,
    /// The output rotation of the bone. Solving the skeleton updates this.
    output_rot_g: Global<UnitQuat>,
}
impl Bone {
    pub fn new(kind: BoneKind, length: f32) -> Self {
        Self {
            kind,
            input_rot_g: None,
            calib_rot_l: kind.calibration_rotation_local(),
            length,
            output_rot_g: Default::default(),
        }
    }

    pub fn input_rotation_mut(&mut self) -> Option<&mut Global<UnitQuat>> {
        self.input_rot_g.as_mut()
    }

    pub fn output_rotation(&self) -> &Global<UnitQuat> {
        &self.output_rot_g
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn length_mut(&mut self) -> &mut f32 {
        &mut self.length
    }

    pub fn calibration_rotation(&self) -> &Local<UnitQuat> {
        &self.calib_rot_l
    }

    pub fn kind(&self) -> BoneKind {
        self.kind
    }
}
