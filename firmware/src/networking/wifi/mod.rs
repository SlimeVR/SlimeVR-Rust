use core::str::FromStr;

use defmt::debug;
use embassy_futures::yield_now;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi, Status, ClientStatus};

use crate::utils;

#[cfg(feature = "esp-wifi")]
#[path = "esp.rs"]
pub mod ඞ;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

pub async fn connect_wifi<W: Wifi>(wifi: &mut W) -> Result<(), W::Error> {
	if !wifi.is_started()? {
		wifi.start()?
	}

	debug!("wifi scanning...");
	let (scan_list, count) = wifi.scan_n()?;
	debug!("found {} APs", count);
	// we yield because scan_n is blocking
	yield_now().await;

	//FIXME: Maybe we should retry scanning until we find the AP
	let ap = scan_list
		.iter()
		.find(|ap| ap.ssid == SSID)
		.expect("Couldn't find the Wi-Fi access point");
	debug!("found {}", SSID);
	let client_config = Configuration::Client(ClientConfiguration {
		ssid: SSID.into(),
		password: PASSWORD.into(),
		bssid: Some(ap.bssid),
		auth_method: ap.auth_method,
		channel: Some(ap.channel)
	});
	wifi.set_configuration(&client_config)?;

	loop {
		if wifi.is_connected()? { break; }
		//FIXME: Maybe a ticker would be better in this case.
		yield_now().await;
	}

	Ok(())
}
