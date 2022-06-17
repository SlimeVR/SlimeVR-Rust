//! An experimental skeletal model for full body tracking (FBT) in VR.
//!
//! Documentation for contributors is available [here][internal rustdoc].
//! For documentation without internal implementation details, please run
//! `cargo doc --all --no-deps --open`
//!
//! [internal rustdoc]: https://thebutlah.github.io/ferrous_slimevr/skeletal_model
//!
//!
//! # Overview
//!
//! This crate provides a well documented, high performance, and very robust model of
//! a human's pose for the purpose of full body tracking in VR. The role of this crate
//! is to ingest partially complete human pose data, often from SlimeVR or Vive
//! trackers, and compute the missing pose data. This gives applications a way of
//! accessing that pose data.
//!
//! This is accomplished via a simple forward-kinematics solver, which also accounts for
//! positional constraints from positional trackers, if any are present.
//!
//! This crate was both inspired by and written for use in [SlimeVR], and hopes to be a
//! better version of the current skeletal model implemented in the [java server].
//!
//! [java server]: https://github.com/SlimeVR/SlimeVR-Server/
//! [SlimeVR]: https://github.com/SlimeVR/
//!
//!
//! # Tracker support
//!
//! Typically, there are two types of full body trackers:
//! - **"3DoF"**: Any tracker with only rotational data. Example: [SlimeVR] trackers
//! - **"6DoF"**: Any tracker with both positional and rotational data. Example: Vive
//!   trackers.
//!
//! As long as you can provide position and/or rotation for some bones, this crate
//! should do the rest.
//!
//!
//! # Scope
//!
//! It is important to note that this crate is *only* the skeletal model, and does not
//! handle networking or anything else necessary for actually reading tracker data. It
//! also does not expose any networked way of acessing the outputs of the skeleton. This
//! enables applications to then build on top of this crate, either through a rust
//! implementation of the SlimeVR server, or by calling this library via any of the
//! various language bindings we hope to add soon. We hope that this will enable the
//! SlimeVR community, as well as other  body tracking
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
    rustdoc::broken_intra_doc_links,
    invalid_doc_attributes,
    unused_import_braces,
    unused
)]

pub mod bone;
pub mod conventions;
mod edge;
mod joint;
mod newtypes;
pub mod prelude;
pub mod skeleton;

pub use crate::skeleton::Skeleton;
