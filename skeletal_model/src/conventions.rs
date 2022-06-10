//! This describes the mathematical conventions we adopt.
//!
//! # Coordinate system
//! We adopt a right hand rule coordinate system - your pointer finger is X, middle
//! finger is Y, thumb is Z. +Y is up, +X axis is right, and -Z is looking "forward".
//! This convention is the same as `nalgebra`'s right handed coordinate system
//! convention, where the view direction is -Z, and is also consistent with other
//! libraries like SteamVR and OpenGL. See [`nalgebra::Vector3::look_at_rh`].
//!
//! # Rotation representation
//! We use quaternions to represent rotations whenever possible. We try not to use
//! quaternions in our implementation to avoid possible gimbal lock issues.

use nalgebra::{Unit, Vector3};

pub const fn up_vec() -> Unit<Vector3<f32>> {
    Vector3::y_axis()
}

pub const fn forward_vec() -> Unit<Vector3<f32>> {
    -Vector3::z_axis()
}

pub const fn right_vec() -> Unit<Vector3<f32>> {
    Vector3::x_axis()
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use nalgebra::Vector3;
    use std::f32::consts::FRAC_PI_2;

    /// Sanity check that euler angles work the way we think they do.
    #[test]
    fn check_euler_matches_matrix() {
        let euler = UnitQuat::from_euler_angles(0, -FRAC_PI_2, 0);
        let matrix = UnitQuat::look_at_rh(&-Vector3::y_axis(), &-Vector3::z_axis());

        assert_eq!(euler, matrix);
    }
}
