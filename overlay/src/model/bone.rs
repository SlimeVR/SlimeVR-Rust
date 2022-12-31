use crate::RGBA;

use eyre::{Result, WrapErr};
use nalgebra::{Isometry3, UnitQuaternion, Vector3};
use ovr_overlay::overlay::{OverlayHandle, OverlayManager};
use ovr_overlay::pose::{Matrix3x4, TrackingUniverseOrigin};
use ovr_overlay::ColorTint;

pub type Isometry = nalgebra::Isometry3<f32>;

#[derive(Debug)]
pub struct Bone {
	overlays: (OverlayHandle, OverlayHandle),
	iso: Isometry,
	color: RGBA,
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
			// Adjusts the rotation of `iso` to have only roll and not pitch, to avoid
			// distortion due to the curvature that SteamVR causes. Also adjusts the
			// translation to place the overlay in the center of the tube instead of
			// the edge
			let mut f = |overlay, mut iso: Isometry, flip: f32| -> Result<()> {
				// Offset for skeleton debugging
				// iso.translation *= Translation3::new(0., 0., -1.);

				// The direction the overlay's y axis points after the transform.
				let y_direction = iso.rotation.transform_vector(&Vector3::y_axis());
				let transform = if y_direction == Vector3::y_axis().into_inner()
					|| y_direction == -Vector3::y_axis().into_inner()
				{
					// just use the existing rotation, there won't be any distortion
					iso.translation.vector += iso.rotation.transform_vector(
						&Vector3::new(0., -self.length / 2.0, -self.radius),
					);
					iso.to_homogeneous().remove_fixed_rows::<1>(3)
				} else {
					// We can freely rotate around `y_direction`, but to avoid
					// distortion we want to have zero pitch, and only roll/yaw. A plane
					// with zero pitch would be the plane between two vectors - one
					// with the global y axis, and one with `y_direction`. Crossing
					// those two vectors gives us the normal of the plane, called
					// `z_direction`.
					let z_direction =
						flip * Vector3::<f32>::y_axis().cross(&y_direction).normalize();

					// Now that we have the y direction and the z direction, we can
					// form the rotation corresponding to this orientation.
					iso.rotation =
						UnitQuaternion::face_towards(&z_direction, &y_direction);
					// let x_basis = y_basis.cross(&z_basis).normalize();

					// Fixes the "center of tube" issue and the "center of overlay"
					// issue
					iso.translation.vector += z_direction * -self.radius;
					iso.translation.vector -= y_direction * self.length / 2.0;

					iso.to_homogeneous().remove_fixed_rows::<1>(3)
				};

				let col_major_3x4 = Matrix3x4::from(&transform);
				mngr.set_transform_absolute(
					overlay,
					TrackingUniverseOrigin::TrackingUniverseStanding,
					&col_major_3x4,
				)
				.wrap_err("Failed to set transform")?;
				Ok(())
			};

			let flipped = {
				let mut rotation = UnitQuaternion::from_axis_angle(
					&Vector3::y_axis(),
					std::f32::consts::PI,
				);
				rotation = self.iso.rotation * rotation;
				Isometry3 {
					rotation,
					translation: self.iso.translation,
				}
			};

			f(self.overlays.0, self.iso, 1.0)?;
			f(self.overlays.1, flipped, -1.0)?;
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

	#[allow(unused)]
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
