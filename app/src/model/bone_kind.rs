use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum BoneKind {
    Head = 0,
    Neck,
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
        Self::Head
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
            Head => &[Neck],
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
            Head => return None,
            Neck => Head,
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
