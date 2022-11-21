use core::str::FromStr;

use defmt::debug;
use embassy_futures::yield_now;
use embedded_svc::wifi::{Wifi, Configuration, ClientConfiguration, AuthMethod};

use crate::utils;

#[cfg(feature = "esp-wifi")]
#[path = "esp.rs"]
pub mod ඞ;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

pub async fn connect_wifi<W: Wifi>(wifi: &mut W) -> Result<(), W::Error> {
	debug!("{:?}", defmt::Debug2Format(&wifi.get_status()));
	AuthMethod::

	debug!("wifi scanning...");
	let (scan_list, count) = wifi.scan_n()?;
	debug!("found {} APs", count);
	// we yield because scan_n is blocking
	yield_now().await;



	Ok(())
} 
