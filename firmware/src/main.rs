#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]
// Needed to use `alloc` + `no_std`
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

mod aliases;
mod bbq_logger;
mod globals;
mod imu;
mod networking;
mod peripherals;
mod utils;

use defmt::debug;
use embassy_executor::{task, Executor};
use embedded_hal::blocking::delay::DelayMs;
use static_cell::StaticCell;

#[cfg(feature = "mcu-nrf52840")]
use cortex_m_rt::entry;
#[cfg(target_arch = "riscv32")]
use riscv_rt::entry;
#[cfg(esp_xtensa)]
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
	#[cfg(all(
		feature = "defmt-bbq",
		any(feature = "log-uart", feature = "log-usb-serial")
	))]
	let bbq = defmt_bbq::init().unwrap();

	self::globals::setup();
	debug!("Booted");
	defmt::trace!("Trace");

	let p = self::peripherals::ඞ::get_peripherals();
	let (bbq_peripheral, mut p) = p.bbq_peripheral();

	p.delay.delay_ms(500u32);
	debug!("Initialized peripherals");

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner.spawn(network_task()).unwrap();
		spawner.spawn(imu_task(p.i2c, p.delay)).unwrap();
		#[cfg(feature = "defmt-bbq")]
		spawner.spawn(logger_task(bbq, bbq_peripheral)).unwrap();
	});
}

#[task]
async fn network_task() {
	networking::network_task().await
}

#[task]
async fn imu_task(
	i2c: crate::aliases::ඞ::I2cConcrete<'static>,
	delay: crate::aliases::ඞ::DelayConcrete,
) {
	debug!("IMU Task!");
	crate::imu::imu_task(i2c, delay).await
}

#[cfg(feature = "defmt-bbq")]
#[task]
async fn logger_task(
	bbq: defmt_bbq::DefmtConsumer,
	logger_peripheral: crate::aliases::ඞ::BbqPeripheralConcrete<'static>,
) {
	crate::bbq_logger::logger_task(bbq, logger_peripheral).await;
}
