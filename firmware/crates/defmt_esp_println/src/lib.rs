//! [`defmt`](https://github.com/knurling-rs/defmt) global logger over
//! [`esp-println`](https://github.com/esp-rs/esp-println)
//!
//! # Usage
//!
//! Import the crate somewhere in your project. For example:
//! ```
//! // src/main.rs
//! use defmt_esp_println as _;
//! ```
//!
//! # Critical section implementation
//!
//! This crate uses [`critical-section`](https://github.com/rust-embedded/critical-section) to ensure only one thread
//! is writing to the buffer at a time. You must import a crate that provides a `critical-section` implementation
//! suitable for the current target. See the `critical-section` README for details.
//!
//! For example, for single-core privileged-mode Cortex-M targets, you can add the following to your Cargo.toml.
//!
//! ```toml
//! [dependencies]
//! cortex-m = { version = "0.7.6", features = ["critical-section-single-core"]}
//! ```

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

cfg_if::cfg_if! {
	if #[cfg(any(feature = "uart", feature = "usb-serial"))] {
		mod esp32_all as maybe_mod;
		use esp32_all::*;
	}
}
