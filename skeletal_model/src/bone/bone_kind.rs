use crate::prelude::*;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::f32::consts::FRAC_PI_2;

/// `BoneKind` describes the fixed relationship between the various types of bones, as
/// well as static, compile-time information about them such as their parents/children,
/// their calibration pose, etc.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum BoneKind {
    Neck = 0,
    Chest,
    Waist,
    Hip,
    ThighL,
    ThighR,
    AnkleL,
    AnkleR,
    FootL,
    FootR,

    UpperArmL,
    UpperArmR,
    ForearmL,
    ForearmR,
    WristL,
    WristR,
}
impl BoneKind {
    /// The `BoneKind` with the largest integer value
    pub const fn max() -> BoneKind {
        BoneKind::WristR
    }
    pub const MAX: BoneKind = Self::max();

    /// The `BoneKind` with the smallest integer value
    pub const fn min() -> BoneKind {
        BoneKind::root()
    }
    pub const MIN: BoneKind = Self::min();

    /// The root bone type.
    pub const fn root() -> Self {
        Self::Neck
    }
    pub const ROOT: BoneKind = Self::root();

    /// Returns the number of different types of bones.
    pub const fn num_types() -> usize {
        BoneKind::max() as usize + 1
    }
    pub const NUM_TYPES: usize = Self::num_types();

    pub const fn children(&self) -> &'static [Self] {
        use BoneKind::*;
        match self {
            Neck => &[Chest, UpperArmL, UpperArmR],
            Chest => &[Waist],
            Waist => &[Hip],
            Hip => &[ThighL, ThighR],
            ThighL => &[AnkleL],
            ThighR => &[AnkleR],
            AnkleL => &[FootL],
            AnkleR => &[FootR],
            FootL => &[],
            FootR => &[],

            UpperArmL => &[ForearmL],
            UpperArmR => &[ForearmR],
            ForearmL => &[WristL],
            ForearmR => &[WristR],
            WristR => &[],
            WristL => &[],
        }
    }

    pub const fn parent(&self) -> Option<BoneKind> {
        use BoneKind::*;
        Some(match self {
            Neck => return None,
            Chest => Neck,
            Waist => Chest,
            Hip => Waist,
            ThighL => Hip,
            ThighR => Hip,
            AnkleL => ThighL,
            AnkleR => ThighR,
            FootL => AnkleL,
            FootR => AnkleR,

            UpperArmL => Neck,
            UpperArmR => Neck,
            ForearmL => UpperArmL,
            ForearmR => UpperArmR,
            WristL => ForearmL,
            WristR => ForearmR,
        })
    }

    pub fn iter() -> std::iter::Map<std::ops::RangeInclusive<u8>, fn(u8) -> BoneKind> {
        (Self::MIN as u8..=Self::MAX as u8).map(|x| x.try_into().unwrap())
    }

    /// Returns the initial calibration pose of the bone. Rotating the up vector by
    /// this rotation would cause it to point in the same target direction as the bone.
    pub const fn calibration_rotation(self) -> Global<UnitQuat> {
        use BoneKind::*;
        Global(match self {
            FootL | FootR => UnitQuat::look_at_rh(&-up_vec(), &forward_vec()),
            _ => UnitQuat::default(),
        })
    }

    /// Returns the initial calibration pose of the bone, as a rotation relative to the
    /// parent bone. See also: [`Self::calibration_rotation`]
    pub const fn calibration_rotation_local(self) -> Local<UnitQuat> {
        let child_rot_g = self.calibration_rotation();
        let parent_rot_g = self.parent().unwrap_or(self).calibration_rotation();
        Local(parent_rot_g.0.rotation_to(&child_rot_g.0))
    }
}
impl TryFrom<u8> for BoneKind {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(())
    }
}
impl TryFrom<usize> for BoneKind {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        FromPrimitive::from_usize(value).ok_or(())
    }
}
impl From<BoneKind> for u8 {
    fn from(other: BoneKind) -> Self {
        other as _
    }
}
impl From<BoneKind> for usize {
    fn from(other: BoneKind) -> Self {
        other as _
    }
}
