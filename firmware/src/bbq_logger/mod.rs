#[cfg(all(feature = "mcu-nrf52840", feature = "log-uart"))]
#[path = "embassy_uart.rs"]
mod ඞ;

#[cfg(all(feature = "mcu-nrf52840", feature = "log-usb-serial"))]
#[path = "embassy_usb.rs"]
mod ඞ;

pub use ඞ::logger_task;
