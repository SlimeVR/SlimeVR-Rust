#![allow(unused)]

pub type Translation = nalgebra::Translation3<f32>;
pub type UnitQuat = nalgebra::UnitQuaternion<f32>;
pub type Point = nalgebra::Point3<f32>;

pub use crate::bone::BoneKind;
pub(crate) use crate::conventions::{forward_vec, right_vec, up_vec};
pub(crate) use crate::newtypes::{Global, Local};
