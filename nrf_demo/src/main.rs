#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;

use nrf52840_hal::pac::UARTE0;
use nrf52840_hal::{gpio, uarte, Uarte};
use panic_halt as _;

use cortex_m_rt::entry;
use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
use nrf52840_hal::prelude::OutputPin;
use nrf52840_hal::{
	gpio::{Level, PushPull},
	Delay,
};
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
	let p = nrf52840_hal::pac::Peripherals::take().unwrap();
	let cp = nrf52840_hal::pac::CorePeripherals::take().unwrap();

	let p0 = nrf52840_hal::gpio::p0::Parts::new(p.P0);
	// let p1 = nrf52840_hal::gpio::p1::Parts::new(p.P1);
	let led = p0.p0_15.into_push_pull_output(Level::Low);
	let delay = Delay::new(cp.SYST);
	let uarte = Uarte::new(
		p.UARTE0,
		uarte::Pins {
			txd: p0.p0_10.into_push_pull_output(gpio::Level::High).degrade(),
			rxd: p0.p0_09.into_floating_input().degrade(),
			cts: None,
			rts: None,
		},
		uarte::Parity::EXCLUDED,
		uarte::Baudrate::BAUD115200,
	);

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner.spawn(main_task(led, delay, uarte)).unwrap();
	});
}

#[task]
async fn main_task(
	mut led: nrf52840_hal::gpio::p0::P0_15<nrf52840_hal::gpio::Output<PushPull>>,
	mut delay: Delay,
	mut uarte: Uarte<UARTE0>,
) {
	loop {
		writeln!(uarte, "Turning on").unwrap();
		// Yes, these should be async, but I was lazy
		delay.delay_ms(1000 as u16);
		led.set_high().expect("Failed to set high");
		delay.delay_ms(1000 as u16);
		writeln!(uarte, "Turning off").unwrap();

		led.set_low().expect("Failed to set low");
		yield_now().await
	}
}
