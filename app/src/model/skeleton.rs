use std::collections::HashMap;

use crate::model::bone::Bone;
use crate::model::BoneKind;
use crate::model::BoneMap;
use crate::RGBA;

use eyre::Result;
use lazy_static::lazy_static;
use ovr_overlay::overlay::OverlayManager;
use stackvec::TryCollect;

pub type BoneArena = BoneMap<Bone>;

lazy_static! {
    static ref DEFAULT_COLORS: BoneMap<RGBA> = {
        use BoneKind::*;
        HashMap::from([
            (Head, RGBA::SILVER),
            (Neck, RGBA::GRAY),
            (Chest, RGBA::OLIVE),
            (Waist, RGBA::LIME),
            (Hip, RGBA::GREEN),
            (ThighL, RGBA::AQUA),
            (ThighR, RGBA::AQUA),
            (AnkleL, RGBA::TEAL),
            (AnkleR, RGBA::TEAL),
            (FootL, RGBA::BLUE),
            (FootR, RGBA::BLUE),
            (UpperArmL, RGBA::RED),
            (UpperArmR, RGBA::RED),
            (ForearmL, RGBA::PURPLE),
            (ForearmR, RGBA::PURPLE),
            (WristL, RGBA::FUCHSIA),
            (WristR, RGBA::FUCHSIA),
        ])
        .try_into()
        .unwrap()
    };
}

/// Builder for the [`Skeleton`].
#[derive(Default)]
pub struct SkeletonBuilder {
    colors: Option<BoneMap<Option<RGBA>>>,
    key: String,
    bone_radius: f32,
}
impl SkeletonBuilder {
    #[allow(dead_code)]
    pub fn build(self, overlay_manager: &mut OverlayManager) -> Result<Skeleton> {
        let colors = if let Some(colors) = self.colors {
            colors
        } else {
            Default::default()
        };
        let colors: BoneMap<RGBA> = colors
            .into_iter()
            .map(|(kind, maybe_color)| (kind, maybe_color.unwrap_or_else(|| DEFAULT_COLORS[kind])))
            .try_collect()
            .unwrap();

        let mut bones = Vec::new();
        for (kind, color) in colors {
            let bone = Bone::new(
                overlay_manager,
                color,
                Default::default(),
                self.key.clone(),
                self.bone_radius,
                0.,
            )?;
            bones.push((kind, bone));
        }
        let bones: BoneArena = bones.into_iter().try_collect().unwrap();
        Ok(Skeleton::new(bones))
    }
}

pub struct Skeleton {
    bones: BoneArena,
}
#[allow(dead_code)]
impl Skeleton {
    pub fn new(bones: BoneArena) -> Self {
        Self { bones }
    }

    pub fn bones(&self) -> &BoneArena {
        &self.bones
    }

    pub fn bones_mut(&mut self) -> &mut BoneArena {
        &mut self.bones
    }
}
