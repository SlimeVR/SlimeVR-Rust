//! Wifi implementation based on `embedded_svc` and `smoltcp`

// NOTE: If you are adding support for a board whose wifi *doesn't* implement
// `embedded_svc`, come open an issue on github! We should discuss what the best
// approach is.

use defmt::{debug, info};
use embassy_futures::yield_now;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use smoltcp::wire::Ipv4Address;

#[cfg(feature = "net-wifi")]
#[path = "esp.rs"]
pub mod ඞ;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
static SERVER_IP: Ipv4Address = Ipv4Address::new(192, 168, 10, 121);
const EXPECTED_NEIGHBOURS: usize = 10;
const WIFI_FIND_RETRIES: usize = 10;

pub async fn wifi_task() -> ! {}

pub async fn connect_wifi<W: Wifi>(
	wifi: &mut W,
	ssid: &str,
	pass: &str,
) -> Result<(), W::Error> {
	if !wifi.is_started()? {
		wifi.start()?
	}

	let mut i = 0;
	let ap = loop {
		i += 1;
		debug!("wifi scanning, retry {}...", i);
		let (mut scan_list, count) = wifi.scan_n::<EXPECTED_NEIGHBOURS>()?;
		debug!("found {} APs", count);

		let pos = scan_list.iter().position(|ap| ap.ssid == ssid);

		if let Some(ap) = pos {
			break scan_list.swap_remove(ap);
		} else if i == WIFI_FIND_RETRIES {
			panic!("Couldn't find SSID {}", ssid);
		}
		// TODO: this also should require a ticker
		yield_now().await;
	};
	info!("found SSID {}", SSID);
	let client_config = Configuration::Client(ClientConfiguration {
		ssid: ssid.into(),
		password: pass.into(),
		bssid: Some(ap.bssid),
		auth_method: ap.auth_method,
		channel: Some(ap.channel),
	});
	wifi.set_configuration(&client_config)?;

	debug!("{:?}", defmt::Debug2Format(&wifi.get_capabilities()?));
	wifi.connect()?;

	loop {
		let res = wifi.is_connected();
		if matches!(res, Ok(true)) {
			break; // connected successfully
		}
		yield_now().await;
	}

	Ok(())
}

/// Stack-based storage for the wifi. Typically built by platform-specific code.
struct Storage<
	const SocketCount: usize,
	const CacheCount: usize,
	const RoutesCount: usize,
> {
	socket_set_entries: [SocketStorage<'a>; SocketCount],
	neighbor_cache_storage: [Option<(IpAddress, Neighbor)>; CacheCount],
	routes_storage: [Option<(IpCidr, Route)>; RoutesCount],
}
