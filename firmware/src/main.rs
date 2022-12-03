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
mod utils;

use defmt::debug;
use embassy_executor::{task, Executor};
use embedded_hal::blocking::delay::DelayMs;
use static_cell::StaticCell;

#[cfg(feature = "mcu-nrf52840")]
use cortex_m_rt::entry;
#[cfg(feature = "mcu-esp32c3")]
use riscv_rt::entry;

#[entry]
fn main() -> ! {
	self::globals::setup();
	debug!("Booted");

	let mut p = self::peripherals::ඞ::get_peripherals();
	debug!("Initialized peripherals");
	p.delay.delay_ms(1000 as u32);

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner.spawn(network_task()).unwrap();
		spawner.spawn(imu_task(p.i2c, p.delay)).unwrap();
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
	crate::imu::imu_task(i2c, delay).await
}
