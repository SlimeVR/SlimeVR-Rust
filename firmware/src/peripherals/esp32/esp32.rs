use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;
use crate::aliases::ඞ::NetConcrete;
use crate::peripherals::Peripherals;

use fugit::RateExtU32;
use paste::paste;

use esp32_hal::{
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
	let p = esp32_hal::peripherals::Peripherals::take();

	let mut system = p.DPORT.split();
	// The ESP-Wifi module requires 240MHz for cpu clock speeed
	let clocks =
		ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

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

	#[cfg(feature = "net-wifi")]
	{
		use esp32_hal::Rng;

		esp_wifi::init_heap();

		let timer_group1 = TimerGroup::new(p.TIMG1, &clocks);
		let rng = Rng::new(p.RNG);
		esp_wifi::initialize(timer_group1.timer0, rng, &clocks)
			.expect("failed to initialize esp-wifi");
	}

	let io = esp32_hal::IO::new(p.GPIO, p.IO_MUX);
	// let hz =
	let i2c = esp32_hal::i2c::I2C::new(
		p.I2C0,
		map_pin!(io, env!("PIN_SDA")),
		map_pin!(io, env!("PIN_SCL")),
		400u32.kHz(),
		&mut system.peripheral_clock_control,
		&clocks,
	);

	let delay = esp32_hal::Delay::new(&clocks);

	#[allow(clippy::let_unit_value)]
	let net = super::init_wifi_stack();

	Peripherals::new().i2c(i2c).delay(delay).net(net)
}
