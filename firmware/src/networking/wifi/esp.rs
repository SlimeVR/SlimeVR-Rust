extern crate alloc;

use core::str;
use defmt::{debug, error, info};
use embedded_svc::ipv4::Interface;
use esp_wifi::{
	create_network_stack_storage, current_millis, network_stack_storage,
	wifi::utils::create_network_interface,
	wifi_interface::{IoError as WifiError, Network, UdpSocket},
};
use smoltcp::socket::UdpPacketMetadata;

use crate::networking::messaging::Signals;

const PORT: u16 = 25565;

pub async fn network_task(signals: &Signals) {
	// TODO: Maybe we should look at the macros in the future for better config
	// (socket_count, neighbour_cache_count, routes_store_count, multicast_store_count)
	let mut storage = create_network_stack_storage!(3, 8, 1, 1);
	let ethernet = create_network_interface(network_stack_storage!(storage));
	let mut wifi = esp_wifi::wifi_interface::Wifi::new(ethernet);
	super::connect_wifi(&mut wifi)
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

	socket.bind(PORT).unwrap();

	let mut buffer = [0u8; 256];
	loop {
		main_loop(&mut socket, signals, &mut buffer).await;
	}
}

async fn main_loop<'s, 'n>(
	socket: &mut UdpSocket<'s, 'n>,
	signals: &Signals,
	buffer: &mut [u8],
) {
	socket.work();
	// Send latest message
	{
		// Grab latest message that we should send
		let msg = signals.latest.wait().await;
		let send_result = socket.send(super::SERVER_IP, PORT, msg.as_bytes());
		// We pass the sent message back so the other side can reuse its buffer
		signals.sent.signal(msg);
		send_result.expect("failed to send");
	}

	match socket.receive(buffer) {
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
