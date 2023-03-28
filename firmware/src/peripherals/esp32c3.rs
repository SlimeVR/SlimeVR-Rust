use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;
use crate::aliases::ඞ::NetConcrete;

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

macro_rules! singleton {
	($t:ty, $val:expr) => {{
		use ::static_cell::StaticCell;
		static STATIC_CELL: StaticCell<$t> = StaticCell::new();
		STATIC_CELL.init($val)
	}};
}

pub fn get_peripherals(
) -> Peripherals<I2cConcrete<'static>, DelayConcrete, (), (), NetConcrete> {
	let p = esp32c3_hal::peripherals::Peripherals::take();

	let mut system = p.SYSTEM.split();
	// The ESP-Wifi module requires 160MHz for cpu clock speeed
	let clocks =
		ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
	// Initialize embassy stuff
	// embassy::init(&clocks);

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

	// Initialize esp-wifi stuff
	#[allow(unused)]
	let net = ();
	#[cfg(feature = "esp-wifi")]
	let net = {
		use embassy_net::{Config, Stack, StackResources};
		use esp_wifi::wifi::{WifiDevice, WifiMode};

		let (wifi_interface, controller) = esp_wifi::wifi::new(WifiMode::Sta);
		let config = Config::Dhcp(Default::default());

		let seed = 1234; // very random, very secure seed

		// Init network stack
		let stack = &*singleton!(
			Stack<WifiDevice>,
			Stack::new(
				wifi_interface,
				config,
				singleton!(StackResources<3>, StackResources::<3>::new()),
				seed
			)
		);
		NetConcrete { controller, stack }
	};

	let io = esp32c3_hal::IO::new(p.GPIO, p.IO_MUX);
	let i2c = esp32c3_hal::i2c::I2C::new(
		p.I2C0,
		map_pin!(io, env!("PIN_SDA")),
		map_pin!(io, env!("PIN_SCL")),
		400u32.kHz(),
		&mut system.peripheral_clock_control,
		&clocks,
	);

	let delay = esp32c3_hal::Delay::new(&clocks);

	Peripherals::new().i2c(i2c).delay(delay).net(net)
}
