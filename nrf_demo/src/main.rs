#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_halt as _;

use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use nrf52840_hal::pac::UARTE0;
use nrf52840_hal::prelude::OutputPin;
use nrf52840_hal::{gpio, uarte, Uarte};

use cortex_m_rt::entry;
// use embassy_executor::{task, Executor};
// use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
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
	let led = p0.p0_15.into_push_pull_output(Level::Low).degrade();
	let delay = Delay::new(cp.SYST);
	let uarte = Uarte::new(
		p.UARTE0,
		uarte::Pins {
			txd: p0.p0_06.into_push_pull_output(gpio::Level::Low).degrade(),
			rxd: p0.p0_08.into_floating_input().degrade(),
			cts: None,
			rts: None,
		},
		uarte::Parity::EXCLUDED,
		uarte::Baudrate::BAUD115200,
	);

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner.spawn(main_task(led, delay, uarte)).unwrap();
	})
}

#[task]
async fn main_task(
	mut led: nrf52840_hal::gpio::Pin<nrf52840_hal::gpio::Output<PushPull>>,
	mut delay: Delay,
	mut uarte: Uarte<UARTE0>,
) {
	// Message must be in SRAM
	const LEN: usize = 13;
	const ON: &[u8; LEN] = b"Turning on \r\n";
	const OFF: &[u8; LEN] = b"Turning off\r\n";
	let mut buf = [0u8; LEN];

	loop {
		// Yes, these should be async, but I was lazy
		buf.copy_from_slice(ON);
		uarte.write(&buf).unwrap();
		led.set_high().expect("Failed to set high");
		delay.delay_ms(1000 as u16);

		buf.copy_from_slice(OFF);
		uarte.write(&buf).unwrap();
		led.set_low().expect("Failed to set low");
		delay.delay_ms(500 as u16);

		yield_now().await
	}
}
