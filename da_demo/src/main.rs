#![no_main]
#![no_std]

// Needed because of the critical-section stuff not getting loaded :P
use cortex_m as _;
use defmt_rtt as _;
use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::delay::Delay;
use da14531_hal::{gpio::p0::Parts,hal::digital::v2::{OutputPin, PinState}};
use defmt::error;

#[entry]
fn main() -> ! {
    let p = da14531_hal::pac::Peripherals::take().unwrap();
    let cp = da14531_hal::pac::CorePeripherals::take().unwrap();

    let p0 = Parts::new(p.GPIO);
    let mut led = p0.p0_09.degrade().into_output(PinState::Low);

    let mut delay = Delay::with_source(cp.SYST, 32000000, cortex_m::peripheral::syst::SystClkSource::Core);
    let mut i = 0;
    loop {
        error!("turning led ON {}", i);
        led.set_high().expect("failed to set HIGH");
        delay.delay_ms(1000u32);

        error!("turning led OFF {}", i);
        led.set_low().expect("failed to set LOW");
        delay.delay_ms(500u32);
        i += 1;
    }
}
