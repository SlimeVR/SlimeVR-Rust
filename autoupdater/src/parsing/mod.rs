// //! Describes the format of the versioning file
// //!
// //! The versioning file describes what the latest version of all the different pieces
// //! of SlimeVR software should be updated to, so that the update can be performed
// //! atomically.
// //!
// //! This file gets deserialized to our [`Components`] datastructure using [`serde`].

mod install_path;
pub use install_path::InstallPath;
