use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;
use crate::aliases::ඞ::NetConcrete;
use crate::peripherals::Peripherals;

use fugit::RateExtU32;
use paste::paste;

use esp32c3_hal::{
	clock::{ClockControl, CpuClock},
	prelude::*,
	timer::TimerGroup,
	Rtc,
};

macro_rules! map_pin {
	($io: ident, $pin: expr) => {
		paste! {
			$io.pins.[<gpio $pin>]
		}
	};
}

pub fn get_peripherals(
) -> Peripherals<I2cConcrete<'static>, DelayConcrete, (), (), NetConcrete> {
	let p = esp32c3_hal::peripherals::Peripherals::take();

	let mut system = p.SYSTEM.split();
	// The ESP-Wifi module requires 160MHz for cpu clock speeed
	let clocks =
		ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

	// Disable the RTC and TIMG watchdog timers
	let timer0 = {
		let mut rtc = Rtc::new(p.RTC_CNTL);
		let timer_group0 = TimerGroup::new(p.TIMG0, &clocks);
		let mut wdt0 = timer_group0.wdt;
		let timer_group1 = TimerGroup::new(p.TIMG1, &clocks);
		let mut wdt1 = timer_group1.wdt;

		rtc.rwdt.disable();
		rtc.swd.disable();
		wdt0.disable();
		wdt1.disable();

		timer_group0.timer0
	};

	// Initialize embassy
	esp32c3_hal::embassy::init(&clocks, timer0);

	let io = esp32c3_hal::IO::new(p.GPIO, p.IO_MUX);
	let i2c = esp32c3_hal::i2c::I2C::new(
		p.I2C0,
		map_pin!(io, env!("PIN_SDA")),
		map_pin!(io, env!("PIN_SCL")),
		400u32.kHz(),
		&clocks,
	);

	let delay = esp32c3_hal::Delay::new(&clocks);

	#[allow(clippy::let_unit_value)]
	let net = super::init_wifi_stack();

	Peripherals::new().i2c(i2c).delay(delay).net(net)
}
