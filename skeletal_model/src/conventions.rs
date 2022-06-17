//! This module describes the mathematical conventions we adopt throughout the skeletal
//! model.
//!
//! # Coordinate system
//! We adopt a right hand rule coordinate system:
//! - Your pointer finger is `+X`, which points "right"
//! - Your middle finger is `+Y`, which points "up"
//! - Your thumb is `+Z`, which points "backward". `-Z` points "forward".
//!
//! This convention is the same as `nalgebra`'s right handed coordinate system
//! convention, where the view direction is `-Z`, and is consistent with other
//! libraries like SteamVR and OpenGL. See also: [`UnitQuat::look_at_rh`].
//!
//! # Rotation representation
//! We use quaternions to represent rotations whenever possible. We try not to use
//! euler angles in our implementation to avoid possible gimbal lock issues.

#[allow(unused)]
use crate::prelude::*;

use nalgebra::{Unit, Vector3};

/// A vector in the "up" or `+Y` direction
pub fn up_vec() -> Unit<Vector3<f32>> {
    Vector3::y_axis()
}

/// A vector in the "forward" or `-Z` direction
pub fn forward_vec() -> Unit<Vector3<f32>> {
    -Vector3::z_axis()
}

/// A vector in the "right" or `+X` direction
pub fn right_vec() -> Unit<Vector3<f32>> {
    Vector3::x_axis()
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use approx::assert_relative_eq;
    use nalgebra::Vector3;
    use std::f32::consts::FRAC_PI_2;

    /// Sanity check that euler angles work the way we think they do.
    #[test]
    fn check_euler_matches_matrix() {
        let euler = UnitQuat::from_euler_angles(0., -FRAC_PI_2, 0.);
        let matrix = UnitQuat::look_at_rh(&-Vector3::y_axis(), &-Vector3::z_axis());

        // TODO: why does this fail?
        assert_relative_eq!(euler, matrix);
    }
}
