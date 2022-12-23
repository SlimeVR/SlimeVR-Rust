extern crate alloc;
use alloc::format;
use core::str;

use defmt::{debug, error, info};
use embassy_futures::yield_now;
use embedded_svc::{ipv4::Interface, wifi::Wifi};
use esp_wifi::{
	create_network_stack_storage, current_millis, network_stack_storage,
	wifi::utils::create_network_interface,
	wifi_interface::{IoError as WifiError, Network},
};
use smoltcp::{
	iface::{Neighbor, Route, SocketStorage},
	socket::UdpPacketMetadata,
	wire::{IpAddress, IpCidr},
};

use super::{PASSWORD, SSID};

const SOCKET_COUNT: usize = 3;
const CACHE_COUNT: usize = 8;
const ROUTES_COUNT: usize = 1;
const MULTICAST_STORE_SIZE: usize = 1;

pub struct WifiStorage {
	socket_set_entries: [SocketStorage; SOCKET_COUNT],
	neighbor_cache_storage: [Option<(IpAddress, Neighbor)>; CACHE_COUNT],
	routes_storage: [Option<(IpCidr, Route)>; ROUTES_COUNT],
	ip_addr: IpCidr,
	ipv4_multicast_storage: [Option<(Ipv4Address, ())>; MULTICAST_STORE_SIZE],
}

pub fn create_wifi_storage() -> WifiStorage {
	let storage = create_network_stack_storage!(
		SOCKET_COUNT,
		CACHE_COUNT,
		ROUTES_COUNT,
		MULTICAST_STORE_SIZE
	);
	WifiStorage {
		socket_set_entries: storage.0,
		neighbor_cache_storage: storage.1,
		routes_storage: storage.2,
		ip_addr: storage.3,
		ipv4_multicast_storage: storage.5,
	}
}

pub fn create_wifi(storage: &mut WifiStorage) -> impl Wifi {
	let ethernet = create_network_interface((
		&mut storage.socket_set_entries,
		&mut storage.neighbor_cache_storage,
		&mut storage.routes_storage,
		&mut [storage.ip_addr],
		&mut storage.ipv4_multicast_storage,
	));
	esp_wifi::wifi_interface::Wifi::new(ethernet)
}

pub async fn network_task() {
	// TODO: Maybe we should look at the macros in the future for better config
	// (socket_count, neighbour_cache_count, routes_store_count, multicast_store_count)
	let mut storage = create_wifi_storage();
	let mut wifi = create_wifi(&mut storage);
	super::connect_wifi(&mut wifi, SSID, PASSWORD)
		.await
		.expect("Couldn't connect to wifi");

	let mut network = Network::new(wifi, current_millis);
	poll_dhcp(&mut network).await;

	// I think its better for each arch to make it's buffers
	let mut rx_buffer = [0u8; 1536];
	let mut tx_buffer = [0u8; 1536];
	let mut rx_meta = [UdpPacketMetadata::EMPTY];
	let mut tx_meta = [UdpPacketMetadata::EMPTY];
	let mut socket = network.get_udp_socket(
		&mut rx_meta,
		&mut rx_buffer,
		&mut tx_meta,
		&mut tx_buffer,
	);

	let mut buffer = [0u8; 256];
	let mut i = 0;
	socket.bind(25565).unwrap();
	loop {
		socket.work();

		socket
			.send(super::SERVER_IP, 25565, format!("i was {i}\n").as_bytes())
			.expect("failed to send");

		match socket.receive(&mut buffer) {
			Ok((len, _addr, _port)) => unsafe {
				info!(
					"Received packet: \"{}\"",
					str::from_utf8_unchecked(&buffer[0..len])
				);
			},
			Err(WifiError::Other(smoltcp::Error::Exhausted)) => {}
			Err(WifiError::Other(e)) => error!("smoltcp error {}", e),
			Err(e) => error!("esp-wifi error {}", defmt::Debug2Format(&e)),
		}
		i += 1;
		yield_now().await
		//Timer::after(Duration::from_millis(1000)).await
	}
}

pub async fn poll_dhcp(net: &mut Network<'_>) {
	// wait for getting an ip address
	debug!("Wait to get an ip address");
	loop {
		if let Err(e) = net.poll_dhcp() {
			debug!("{:?}", defmt::Debug2Format(&e));
		}

		net.work();

		if net.is_iface_up() {
			info!("got ip {:?}", defmt::Debug2Format(&net.get_ip_info()));
			break;
		}
	}
}
