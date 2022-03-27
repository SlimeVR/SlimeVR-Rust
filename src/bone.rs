use std::collections::HashMap;

use crate::RGBA;

use eyre::{Result, WrapErr};
use lazy_static::lazy_static;
use ovr_overlay::overlay::{OverlayHandle, OverlayManager};
use ovr_overlay::Context;

pub type Isometry = nalgebra::Isometry3<f32>;

pub struct Bone {
    overlays: (OverlayHandle, OverlayHandle),
    iso: Isometry,
    color: RGBA,
    keys: (String, String),
    radius: f32,
    length: f32,
}
impl Bone {
    pub fn new(
        mngr: &mut OverlayManager,
        color: RGBA,
        isometry: Isometry,
        key: String,
        radius: f32,
        length: f32,
    ) -> Result<Self> {
        let keys = (format!("{key}_0"), format!("{key}_1"));

        let mut init_overlay = |key: &str| -> Result<OverlayHandle> {
            let overlay = mngr
                .create_overlay(key, key)
                .wrap_err("Failed to create overlay")?;
            mngr.set_curvature(overlay, 1.)?;

            let pixel_data = vec![color.red, color.green, color.blue, color.alpha];
            mngr.set_raw_data(overlay, pixel_data, 1, 1, 4)
                .wrap_err("Failed to set texture data")?;

            let width = radius * 2. * std::f32::consts::PI;
            mngr.set_width(overlay, 2. * std::f32::consts::PI * radius)
                .wrap_err("Failed to set radius")?;

            Ok(overlay)
        };

        let overlays = (init_overlay(&keys.0)?, init_overlay(&keys.1)?);

        Ok(Self {
            overlays,
            keys,
            iso: isometry,
            radius,
            length,
            color,
        })
    }

    pub fn update_render(&self, mngr: &mut OverlayManager<'_>) -> Result<()> {
        // TODO: Set position
        // TODO: Set rotation
        // TODO: Set length

        Ok(())
    }

    pub fn set_iso(&mut self, iso: Isometry) {
        self.iso = iso;
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn set_visibility(&self, mngr: &mut OverlayManager, is_visible: bool) -> eyre::Result<()> {
        mngr.set_visibility(self.overlays.0, is_visible)
            .and_then(|_| mngr.set_visibility(self.overlays.1, is_visible))
            .wrap_err("Failed to show overlay")
    }
}
