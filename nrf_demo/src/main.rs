#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;

use cortex_m::peripheral::SCB;
use nrf52840_hal::pac::UARTE0;
use nrf52840_hal::{gpio, uarte, Uarte};
use panic_halt as _;

use cortex_m_rt::{entry, pre_init};
// use embassy_executor::{task, Executor};
// use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
use nrf52840_hal::prelude::OutputPin;
use nrf52840_hal::{
	gpio::{Level, PushPull},
	Delay,
};
use static_cell::StaticCell;

// according to gabu#6113 on the nice!nano discord, normally the chip expects the
// interrupt table at 0x0000. But since our code starts at 0x1000, our interrupt table
// is there. So we must set the value in the register.
// #[pre_init]
// unsafe fn before_main() {
// 	(*SCB::PTR).vtor.write(0x1000);
// }

#[entry]
fn main() -> ! {
	let p = nrf52840_hal::pac::Peripherals::take().unwrap();
	let cp = nrf52840_hal::pac::CorePeripherals::take().unwrap();

	let p0 = nrf52840_hal::gpio::p0::Parts::new(p.P0);
	// let p1 = nrf52840_hal::gpio::p1::Parts::new(p.P1);
	let mut led = p0.p0_15.into_push_pull_output(Level::Low).degrade();
	let mut delay = Delay::new(cp.SYST);
	// let mut uarte = Uarte::new(
	// 	p.UARTE0,
	// 	uarte::Pins {
	// 		txd: p0.p0_06.into_push_pull_output(gpio::Level::Low).degrade(),
	// 		rxd: p0.p0_08.into_floating_input().degrade(),
	// 		cts: None,
	// 		rts: None,
	// 	},
	// 	uarte::Parity::EXCLUDED,
	// 	uarte::Baudrate::BAUD115200,
	// );

	// Message must be in SRAM
	// let mut buf = [0u8; 64];
	loop {
		// buf.copy_from_slice(b"Turning on\r\n");
		// uarte.write(&buf).unwrap();
		// Yes, these should be async, but I was lazy
		delay.delay_ms(2000 as u16);
		led.set_high().expect("Failed to set high");
		delay.delay_ms(1000 as u16);
		// buf.copy_from_slice(b"Turning off\r\n");
		// uarte.write(&buf).unwrap();

		led.set_low().expect("Failed to set low");
	}

	// static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	// EXECUTOR.init(Executor::new()).run(move |spawner| {
	// 	spawner.spawn(main_task(led, delay, uarte)).unwrap();
	// });
}

// #[task]
// async fn main_task(
// 	mut led: nrf52840_hal::gpio::Pin<nrf52840_hal::gpio::Output<PushPull>>,
// 	mut delay: Delay,
// 	mut uarte: Uarte<UARTE0>,
// ) {
// 	// Message must be in SRAM
// 	let mut buf = [0u8; 64];
// 	loop {
// 		buf.copy_from_slice(b"Turning on\r\n");
// 		uarte.write(&buf).unwrap();
// 		// Yes, these should be async, but I was lazy
// 		delay.delay_ms(4000 as u16);
// 		// led.set_high().expect("Failed to set high");
// 		delay.delay_ms(1000 as u16);
// 		buf.copy_from_slice(b"Turning off\r\n");
// 		uarte.write(&buf).unwrap();
//
// 		// led.set_low().expect("Failed to set low");
// 		yield_now().await
// 	}
// }
