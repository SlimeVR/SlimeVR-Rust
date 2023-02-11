//! An experimental skeletal model for full body tracking (FBT) in VR.
//!
//! Documentation for contributors is available [here][internal rustdoc].
//! For documentation without internal implementation details, please run
//! `cargo doc --all --no-deps --open`
//!
//! [internal rustdoc]: https://slimevr.github.io/SlimeVR-Rust/skeletal_model
//!
//!
//! # Overview
//!
//! This crate provides a well documented, high performance, and very robust model of
//! a human's pose for the purpose of full body tracking in VR. The role of this crate
//! is to take in partially complete human pose data, often from SlimeVR or Vive
//! trackers, and compute the missing pose data. This gives applications a way of
//! accessing that pose data.
//!
//! This is accomplished via a simple [forward-kinematics][fk] solver, which also
//! accounts for positional constraints from positional trackers, if any are present.
//!
//! This crate was both inspired by and written for use in [SlimeVR], and hopes to be a
//! better version of the current skeletal model implemented in the [Java server].
//!
//! [Java server]: https://github.com/SlimeVR/SlimeVR-Server/
//! [SlimeVR]: https://github.com/SlimeVR/
//! [fk]: https://wulverblade.com/advanced-animation-techniques-fk-ik/
//!
//!
//! # Tracker support
//!
//! Typically, there are two types of full body trackers:
//! - **"3DoF"**: Any tracker with only rotational data. Example: [SlimeVR] trackers
//! - **"6DoF"**: Any tracker with both positional and rotational data. Example: [Vive
//!   trackers](https://www.vive.com/us/accessory/tracker3/).
//!
//! As long as you can provide position and/or rotation for some trackers, this crate
//! should do the rest.
//!
//!
//! # Scope
//!
//! This crate is *only* the skeletal model, and does not handle networking or anything
//! else necessary for actually reading tracker data. It also does not expose any
//! networked way of accessing the outputs of the skeleton.
//!
//! This enables applications to then build on top of this crate, either through a Rust
//! implementation of the SlimeVR server, or by calling this library via any of the
//! various language bindings we hope to add soon.
//!
//! We hope that this will enable the SlimeVR community, as well as other full body
//! tracking solutions, to build on our work.
//!
//!
//! # Documentation on Internals
//!
//! For an explanation of the design and mathematics of the skeletal model, see
//! the [`skeleton`] module.
//!
//! For an explanation of the mathematical conventions adopted in the codebase,
//! see the [`conventions`] module.

// These set linter options
#![deny(
	invalid_doc_attributes,
	rustdoc::broken_intra_doc_links,
	rustdoc::private_intra_doc_links,
	unused_import_braces
)]

pub mod bone;
pub mod conventions;
mod newtypes;
pub mod skeleton;

pub use crate::bone::{BoneKind, BoneMap};
pub use crate::skeleton::Skeleton;

#[allow(unused)]
pub(crate) use crate::conventions::{forward_vec, right_vec, up_vec};
#[allow(unused)]
pub(crate) use crate::newtypes::{Global, Local};

pub type Translation = nalgebra::Translation3<f32>;
pub type UnitQuat = nalgebra::UnitQuaternion<f32>;
pub type Point = nalgebra::Point3<f32>;
