//! Implementations for `defmt-bbq` based loggers.
//!
//! `defmt-bbq` is a logger that takes any `defmt` logged data and enqueues it in a
//! special concurrent queue from the `bbqueue` library. We then run an async task to
//! repeatedly pop from that queue and send that data across some hardware mechanism,
//! such as USB serial or UART.

#[cfg(not(bbq))]
compile_error!("This module will only ever be compiled with the `bbq` feature");

#[cfg(all(feature = "log-uart"))]
#[path = "embassy_uart.rs"]
pub mod ඞ;

#[cfg(all(bbq, feature = "log-usb-serial"))]
#[path = "embassy_usb.rs"]
pub mod ඞ;
