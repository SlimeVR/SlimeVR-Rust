#[cfg(feature = "mcu-esp32c3")]
#[path = "esp32c3.rs"]
mod ඞ;

#[cfg(feature = "mcu-esp32")]
#[path = "esp32.rs"]
mod ඞ;

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

fn init_wifi() -> NetConcrete {
	#[allow(unused_variables)]
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

	net
}
