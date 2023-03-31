//! An implementation of SlimeVR firmware, written in Rust.

#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]
// Needed to use `alloc` + `no_std`
#![feature(alloc_error_handler)]
// We want to do some floating point math at compile time
#![feature(const_fn_floating_point_arithmetic)]
// We need async traits for efficient yet generic IMUs
#![feature(async_fn_in_trait)]
#![deny(unsafe_op_in_unsafe_fn)]

load_dotenv::try_load_dotenv!();

mod aliases;
mod globals;
mod imu;
mod networking;
mod peripherals;
mod utils;

#[cfg(bbq)]
mod bbq_logger;

use defmt::debug;
use embassy_executor::Executor;
use static_cell::StaticCell;

#[cfg(cortex_m)]
use cortex_m_rt::entry;
#[cfg(feature = "mcu-esp32")]
use esp32_hal::entry;
#[cfg(feature = "mcu-esp32c3")]
use esp32c3_hal::entry;

#[entry]
fn main() -> ! {
	use crate::imu::Quat;
	use crate::networking::protocol::Packets;
	use crate::utils::Unreliable;
	use embedded_hal::blocking::delay::DelayMs;

	#[cfg(bbq)]
	let bbq = defmt_bbq::init().unwrap();

	self::globals::setup();
	debug!("Booted");
	defmt::trace!("Trace");

	let p = self::peripherals::ඞ::get_peripherals();
	#[allow(unused)]
	let (bbq_peripheral, mut p) = p.bbq_peripheral();

	p.delay.delay_ms(500u32);
	debug!("Initialized peripherals");

	static PACKETS: StaticCell<Packets> = StaticCell::new();
	let packets: &'static Packets = PACKETS.init(Packets::new());

	static QUAT: StaticCell<Unreliable<Quat>> = StaticCell::new();
	let quat: &'static Unreliable<Quat> = QUAT.init(Unreliable::new());

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |s| {
		s.spawn(crate::networking::protocol::control_task(packets, quat))
			.unwrap();
		#[allow(clippy::unit_arg)]
		s.spawn(crate::networking::network_task(s, packets, p.net))
			.unwrap();
		s.spawn(crate::imu::imu_task(quat, p.i2c, p.delay)).unwrap();

		#[cfg(bbq)]
		s.spawn(logger_task(bbq, bbq_peripheral)).unwrap();
	});
}

#[cfg(bbq)]
#[embassy_executor::task]
async fn logger_task(
	bbq: defmt_bbq::DefmtConsumer,
	logger_peripheral: crate::aliases::ඞ::BbqPeripheralConcrete<'static>,
) {
	crate::bbq_logger::ඞ::logger_task(bbq, logger_peripheral).await;
}
