use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;

use fugit::RateExtU32;

use esp32_hal::{
	clock::{ClockControl, CpuClock},
	pac,
	prelude::*,
	timer::TimerGroup,
	Rtc,
};

pub fn get_peripherals() -> Peripherals<I2cConcrete<'static>, DelayConcrete> {
	let p = pac::Peripherals::take().unwrap();

	let mut system = p.DPORT.split();
	// The ESP-Wifi module requires 240MHz for cpu clock speeed
	let clocks =
		ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
	// Initialize embassy stuff
	// embassy::init(&clocks);

	// Disable the RTC and TIMG watchdog timers
	let timer0 = {
		let mut rtc = Rtc::new(p.RTC_CNTL);
		let timer_group0 = TimerGroup::new(p.TIMG0, &clocks);
		let mut wdt0 = timer_group0.wdt;
		//let timer_group1 = TimerGroup::new(p.TIMG1, &clocks);
		//let mut wdt1 = timer_group1.wdt;

		rtc.rwdt.disable();
		wdt0.disable();
		//wdt1.disable();

		timer_group0.timer0
	};

	// Initialize embassy
	esp32_hal::embassy::init(&clocks, timer0);

	// Initialize esp-wifi stuff
	#[cfg(feature = "net-wifi")]
	{
		esp_wifi::init_heap();
		let timerg = TimerGroup::new(p.TIMG1, &clocks);
		esp_wifi::initialize(timerg.timer0, p.RNG, &clocks)
			.expect("failed to initialize esp-wifi");
	}

	let io = esp32_hal::IO::new(p.GPIO, p.IO_MUX);
	// let hz =
	let i2c = esp32_hal::i2c::I2C::new(
		p.I2C0,
		io.pins.gpio21,
		io.pins.gpio22,
		400u32.kHz(),
		&mut system.peripheral_clock_control,
		&clocks,
	);

	let delay = esp32_hal::Delay::new(&clocks);
	Peripherals::new().i2c(i2c).delay(delay)
}
