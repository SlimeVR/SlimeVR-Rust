use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;

use fugit::RateExtU32;

use esp32c3_hal::{
	clock::{ClockControl, CpuClock},
	prelude::*,
	timer::TimerGroup,
	Rtc,
};

pub fn get_peripherals() -> Peripherals<I2cConcrete<'static>, DelayConcrete> {
	let p = esp32c3_hal::pac::Peripherals::take().unwrap();

	let mut system = p.SYSTEM.split();
	// The ESP-Wifi module requires 160MHz for cpu clock speeed
	let clocks =
		ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
	// Initialize embassy stuff
	// embassy::init(&clocks);

	// Disable the RTC and TIMG watchdog timers
	{
		let mut rtc = Rtc::new(p.RTC_CNTL);
		let timer_group0 = TimerGroup::new(p.TIMG0, &clocks);
		let mut wdt0 = timer_group0.wdt;
		let timer_group1 = TimerGroup::new(p.TIMG1, &clocks);
		let mut wdt1 = timer_group1.wdt;

		rtc.rwdt.disable();
		rtc.swd.disable();
		wdt0.disable();
		wdt1.disable();
	}

	// Initialize esp-wifi stuff
	#[cfg(feature = "net-wifi")]
	{
		use esp32c3_hal::systimer::SystemTimer;

		esp_wifi::init_heap();
		let systimer = SystemTimer::new(p.SYSTIMER);
		esp_wifi::initialize(systimer.alarm0, p.RNG, &clocks)
			.expect("failed to initialize esp-wifi");
	}

	let io = esp32c3_hal::IO::new(p.GPIO, p.IO_MUX);
	// let hz =
	let i2c = esp32c3_hal::i2c::I2C::new(
		p.I2C0,
		io.pins.gpio10,
		io.pins.gpio8,
		400u32.kHz(),
		&mut system.peripheral_clock_control,
		&clocks,
	)
	.expect("Failed to set up i2c");

	let delay = esp32c3_hal::Delay::new(&clocks);
	Peripherals::new().i2c(i2c).delay(delay)
}
