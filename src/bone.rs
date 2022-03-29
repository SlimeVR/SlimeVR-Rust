use crate::RGBA;

use eyre::{eyre, Result, WrapErr};
use ovr_overlay::overlay::{OverlayHandle, OverlayManager};
use ovr_overlay::pose::{Matrix3x4, TrackingUniverseOrigin};
use ovr_overlay::ColorTint;

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
            mngr.set_curvature(overlay, 1.)
                .wrap_err("Failed to set curvature")?;
            mngr.set_raw_data(overlay, &[255u8; 4], 1, 1, 4)
                .wrap_err("Failed to set raw data")?;

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
            is_visible: false,
        })
    }

    pub fn update_render(&self, mngr: &mut OverlayManager<'_>) -> Result<()> {
        // Set Color
        {
            fn f(color: u8) -> f32 {
                color as f32 / 255.
            }
            let tint = ColorTint {
                r: f(self.color.r),
                g: f(self.color.g),
                b: f(self.color.b),
                a: f(self.color.a),
            };
            mngr.set_tint(self.overlays.0, tint)
                .and_then(|_| mngr.set_tint(self.overlays.1, tint))
                .wrap_err("Failed to set color")?;
        }

        // Set width and height
        {
            let mut f = |overlay| -> Result<()> {
                mngr.set_width(overlay, self.circumference())
                    .wrap_err("Failed to set radius")?;
                let aspect = self.circumference() / self.length;
                mngr.set_texel_aspect(overlay, aspect)
                    .wrap_err("Failed to set texture aspect ratio")?;

                Ok(())
            };

            f(self.overlays.0)?;
            f(self.overlays.1)?;
        }

        mngr.set_visibility(self.overlays.0, self.is_visible)
            .and_then(|_| mngr.set_visibility(self.overlays.1, self.is_visible))
            .wrap_err("Failed to show overlay")?;

        // Set transform
        {
            let mut f = |overlay, iso: &Isometry| -> Result<()> {
                let homo: nalgebra::Matrix4<f32> = iso.to_homogeneous();
                let col_major_3x4 = Matrix3x4::from(homo.fixed_rows::<3>(0));
                mngr.set_transform_absolute(
                    overlay,
                    TrackingUniverseOrigin::TrackingUniverseStanding,
                    &col_major_3x4,
                )
                .wrap_err("Failed to set transform")?;
                Ok(())
            };

            f(self.overlays.0, &self.iso)?;
            f(self.overlays.1, &self.iso)?;
        }

        Ok(())
    }

    pub fn set_isometry(&mut self, isometry: Isometry) {
        self.iso = isometry;
    }

    pub fn set_length(&mut self, length: f32) {
        assert!(length >= 0., "Length must be positive");
        self.length = length;
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
