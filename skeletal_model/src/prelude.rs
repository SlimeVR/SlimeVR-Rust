pub use crate::bone::BoneKind;

use derive_more::{From, Into};

pub type Translation = nalgebra::Translation3<f32>;
pub type UnitQuat = nalgebra::UnitQuaternion<f32>;

/// Private helper trait to bound the types that can go in [`Global`] or [`Local`].
trait TTransform {}
impl TTransform for Translation {}
impl TTransform for UnitQuat {}

/// A newtype on `T` that indicates that it is a global transform.
#[derive(From, Into)]
pub struct Global<T: TTransform>(pub T);

/// A newtype on `T` that indicates that it is a local transform.
#[derive(From, Into)]
pub struct Local<T: TTransform>(pub T);

pub use crate::conventions::{forward_vec, right_vec, up_vec};
