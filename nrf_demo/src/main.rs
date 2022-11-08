#![no_main]
#![no_std]

use nrf52840_hal::{gpio::Level, prelude::OutputPin, Delay};
use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    let p = nrf52840_hal::pac::Peripherals::take().unwrap();
    let cp = nrf52840_hal::pac::CorePeripherals::take().unwrap();

    let p0 = nrf52840_hal::gpio::p0::Parts::new(p.P0);
    let mut led = p0.p0_15.into_push_pull_output(Level::Low);
    let mut delay = Delay::new(cp.SYST);

    loop {
        delay.delay_ms(1000 as u16);
        led.set_high().expect("Failed to set high");
        delay.delay_ms(1000 as u16);
        led.set_low().expect("Failed to set low");
    }
}
