use std::collections::HashMap;

use crate::RGBA;

use eyre::{eyre, Result, WrapErr};
use lazy_static::lazy_static;
use ovr_overlay::overlay::{OverlayHandle, OverlayManager};
use ovr_overlay::Context;

pub type Isometry = nalgebra::Isometry3<f32>;

const WIDTH_PIXELS: usize = 10;
const MAX_HEIGHT_PIXELS: usize = 10_000;

pub struct Bone {
    overlays: (OverlayHandle, OverlayHandle),
    iso: Isometry,
    color: RGBA,
    keys: (String, String),
    radius: f32,
    length: f32,
    is_visible: bool,
    pixel_buf: Box<[u8]>,
}
impl Bone {
    pub fn new(
        mngr: &mut OverlayManager,
        color: RGBA,
        isometry: Isometry,
        key: String,
        radius: f32, // meters
        length: f32, // meters
    ) -> Result<Self> {
        let keys = (format!("{key}_0"), format!("{key}_1"));

        let mut init_overlay = |key: &str| -> Result<OverlayHandle> {
            let overlay = mngr
                .create_overlay(key, key)
                .wrap_err("Failed to create overlay")?;
            mngr.set_curvature(overlay, 1.)?;

            Ok(overlay)
        };

        let overlays = (init_overlay(&keys.0)?, init_overlay(&keys.1)?);

        let pixel_buf = vec![color.red, color.green, color.blue, color.alpha]
            .repeat(WIDTH_PIXELS * MAX_HEIGHT_PIXELS)
            .into_boxed_slice();

        Ok(Self {
            overlays,
            keys,
            iso: isometry,
            radius,
            length,
            color,
            is_visible: false,
            pixel_buf,
        })
    }

    pub fn update_render(&self, mngr: &mut OverlayManager<'_>) -> Result<()> {
        mngr.set_width(self.overlays.0, self.circumference())
            .and_then(|_| mngr.set_width(self.overlays.1, self.circumference()))
            .wrap_err("Failed to set radius")?;

        mngr.set_visibility(self.overlays.0, self.is_visible)
            .and_then(|_| mngr.set_visibility(self.overlays.1, self.is_visible))
            .wrap_err("Failed to show overlay")?;

        // set height
        {
            let pixel_height = self.pixel_height().unwrap();
            let slice = &self.pixel_buf[..WIDTH_PIXELS * 4 * pixel_height];
            mngr.set_raw_data(self.overlays.0, slice, WIDTH_PIXELS, pixel_height, 4)
                .and_then(|_| {
                    mngr.set_raw_data(self.overlays.1, slice, WIDTH_PIXELS, pixel_height, 4)
                })
                .wrap_err("Failed to set raw texture data")?;
        }

        // TODO: Set position
        // TODO: Set rotation
        // TODO: Set length

        Ok(())
    }

    /// Returns the height in pixels, or `None` if it exceeds the max height
    fn pixel_height(&self) -> Option<usize> {
        let pixel_height = self.length * WIDTH_PIXELS as f32 / self.circumference();
        let pixel_height = f32::round(pixel_height) as usize;

        if pixel_height > MAX_HEIGHT_PIXELS {
            None
        } else {
            Some(pixel_height)
        }
    }

    pub fn set_isometry(&mut self, isometry: Isometry) {
        self.iso = isometry;
    }

    pub fn set_length(&mut self, length: f32) -> Result<()> {
        assert!(length >= 0., "Length must be positive");
        let old_len = self.length;
        self.length = length;
        if self.pixel_height().is_none() {
            self.length = old_len;
            return Err(eyre!(
                "length too long! Maximum length is {}",
                MAX_HEIGHT_PIXELS
            ));
        }
        Ok(())
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn circumference(&self) -> f32 {
        2. * std::f32::consts::PI * self.radius
    }

    pub fn set_visibility(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }
}
