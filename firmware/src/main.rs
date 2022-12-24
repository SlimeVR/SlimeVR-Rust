//! An implementation of SlimeVR firmware, written in Rust.

#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]
// Needed to use `alloc` + `no_std`
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

mod aliases;
mod globals;
mod imu;
mod networking;
mod peripherals;
mod serialization;
mod utils;

#[cfg(bbq)]
mod bbq_logger;

use defmt::debug;
use embassy_executor::{task, Executor};
use embedded_hal::blocking::delay::DelayMs;
use imu::Quat;
use networking::messaging::Signal;
use static_cell::StaticCell;

#[cfg(cortex_m)]
use cortex_m_rt::entry;
#[cfg(riscv)]
use riscv_rt::entry;
#[cfg(esp_xtensa)]
use xtensa_lx_rt::entry;

use crate::networking::messaging::Signals;

#[entry]
fn main() -> ! {
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

	static MESSAGE_SIGNALS: StaticCell<Signals> = StaticCell::new();
	let message_signals: &'static Signals = MESSAGE_SIGNALS.init(Signals::new());

	static QUAT_SIGNAL: StaticCell<Signal<Quat>> = StaticCell::new();
	let quat_signal: &'static Signal<Quat> = QUAT_SIGNAL.init(Signal::new());

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner
			.spawn(serialize_task(message_signals, quat_signal))
			.unwrap();
		spawner.spawn(network_task(message_signals)).unwrap();
		spawner
			.spawn(imu_task(quat_signal, p.i2c, p.delay))
			.unwrap();
		#[cfg(bbq)]
		spawner.spawn(logger_task(bbq, bbq_peripheral)).unwrap();
	});
}

#[task]
async fn serialize_task(
	msg_signals: &'static Signals,
	quat_signal: &'static Signal<Quat>,
) {
	debug!("Serialize task!");
	crate::serialization::serialize_task(msg_signals, quat_signal).await
}

#[task]
async fn network_task(msg_signals: &'static Signals) {
	debug!("Network task!");
	crate::networking::network_task(msg_signals).await
}

#[task]
async fn imu_task(
	quat_signal: &'static Signal<Quat>,
	i2c: crate::aliases::ඞ::I2cConcrete<'static>,
	delay: crate::aliases::ඞ::DelayConcrete,
) {
	debug!("IMU Task!");
	crate::imu::imu_task(quat_signal, i2c, delay).await
}

#[cfg(bbq)]
#[task]
async fn logger_task(
	bbq: defmt_bbq::DefmtConsumer,
	logger_peripheral: crate::aliases::ඞ::BbqPeripheralConcrete<'static>,
) {
	crate::bbq_logger::ඞ::logger_task(bbq, logger_peripheral).await;
}
