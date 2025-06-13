#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
mod ඞ;

#[cfg(feature = "mcu-esp32")]
#[path = "esp32.rs"]
mod ඞ;

use esp32c3_hal::clock::ClockControl;
use esp_wifi::EspWifiInitFor;
pub use ඞ::get_peripherals;

use crate::aliases::ඞ::NetConcrete;

#[allow(unused_macros)]
macro_rules! singleton {
	($t:ty, $val:expr) => {{
		use ::static_cell::StaticCell;
		static STATIC_CELL: StaticCell<$t> = StaticCell::new();
		STATIC_CELL.init($val)
	}};
}

/// Initializes the esp-wifi controller and stack, but does not initialize
/// esp-wifi itself.
fn init_wifi_stack() -> NetConcrete {
	#[cfg(feature = "net-wifi")]
	{
		use embassy_net::{Config, Stack, StackResources};
		use esp32c3_hal::system::SystemExt;
		use esp_wifi::wifi::{WifiDevice, WifiMode};
		// TODO: don't do this lmao
		let p = unsafe { esp32c3_hal::peripherals::Peripherals::steal() };
		let rng = esp32c3_hal::rng::Rng::new(p.RNG);

		let system = p.SYSTEM.split();
		let clocks = ClockControl::max(system.clock_control).freeze();
		let timer = esp32c3_hal::systimer::SystemTimer::new(p.SYSTIMER).alarm0;

		let initialization = esp_wifi::initialize(
			EspWifiInitFor::Wifi,
			timer,
			rng,
			system.radio_clock_control,
			&clocks,
		)
		.expect("Failed to initialize esp wifi");

		let (wifi_interface, controller) =
			esp_wifi::wifi::new_with_mode(&initialization, p.WIFI, WifiMode::Sta)
				.expect("failed to create new wifi");
		let config = Config::dhcpv4(Default::default());

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
	}
}
