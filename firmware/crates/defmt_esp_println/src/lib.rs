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

#[defmt::global_logger]
struct Logger;

// Much of this code is based on https://github.com/knurling-rs/defmt/blob/ce350f767ffe44bc12513e411a0440c47d5ba0f4/firmware/defmt-rtt/src/lib.rs
use atomic_polyfill::AtomicBool;
use core::sync::atomic::Ordering;

// Global lock on logger
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState =
	critical_section::RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
	fn acquire() {
		// Safety: Must be paired with a call to `release()`
		let restore = unsafe { critical_section::acquire() };

		if TAKEN.load(Ordering::Relaxed) {
			panic!("defmt logger taken reentrantly")
		}
		TAKEN.store(true, Ordering::Relaxed);

		// Safety: accessing static mut is ok in critical section
		unsafe {
			CS_RESTORE = restore;
			ENCODER.start_frame(do_write);
		};
	}

	unsafe fn flush() {
		// By default we already block so this is a no-op
	}

	unsafe fn release() {
		// Safety: accessing static mut is ok in critical section
		unsafe { ENCODER.end_frame(do_write) };

		TAKEN.store(false, Ordering::Relaxed);

		// Safety: accessing static mut is ok in critical section. Also releasing.
		unsafe {
			critical_section::release(CS_RESTORE);
		}
	}

	unsafe fn write(bytes: &[u8]) {
		// Safety: accessing static mut is ok in critical section
		unsafe { ENCODER.write(bytes, do_write) }
	}
}

fn do_write(bytes: &[u8]) {
	// Unfortunately esp-println only lets us print &str:
	// https://github.com/esp-rs/esp-println/issues/19
	use core::fmt::Write;

	// This is really evil, definitely unsafe, and potentially unsound if esp-println were to change.
	// Hopefully it is fine until we can implement bytes printing in esp-println.
	let unsound_str = unsafe { core::str::from_utf8_unchecked(bytes) };
	// Error discarded because printing is best-effort
	let _discard_err = esp_println::Printer.write_str(unsound_str);
}
