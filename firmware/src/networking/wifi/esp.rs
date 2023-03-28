use defmt::{debug, info, trace, warn};
use embassy_executor::{task, Spawner};
use embassy_futures::select::{select, Either};
use embassy_net::udp::{Error as UdpError, UdpSocket};
use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use firmware_protocol::Packet;
use smoltcp::socket::udp::PacketMetadata as UdpPacketMetadata;
use smoltcp::wire::IpEndpoint;
// use esp_wifi::{
// 	create_network_stack_storage, current_millis, network_stack_storage,
// 	wifi::utils::create_network_interface,
// 	wifi_interface::{IoError as WifiError, Network, UdpSocket},
// };
// use smoltcp::{socket::UdpPacketMetadata, wire::Ipv4Address};

use crate::aliases::ඞ::NetConcrete;
use crate::networking::protocol::Packets;

// SlimeVR default UDP port on both sides of connection
const PORT: u16 = 6969;

pub async fn network_task(
	spawner: Spawner,
	packets: &Packets,
	mut net: NetConcrete,
) -> ! {
	spawner.spawn(wifi_stack_task(net.stack)).unwrap();

	connect_to_wifi(&mut net).await;
	let stack = net.stack;
	let ip = stack
		.config()
		.expect("TODO: Can getting the stack config fail after connecting?")
		.address;
	info!("Connected to wifi, ip: {}", defmt::Debug2Format(&ip));

	let mut server_ip = None; // We don't know the server ip yet.

	// Buffer size of 1536 matches modern MTU sizes and is more than enough for the SlimeVR protocol
	// Unfortunately esp-wifi won't let us access the underlying tx/rx buffer. Unecessary copy here
	let mut buffer = [0; 1536];
	let mut rx_buffer = [0u8; 1536];
	let mut tx_buffer = [0u8; 1536];
	let mut rx_meta = [UdpPacketMetadata::EMPTY];
	let mut tx_meta = [UdpPacketMetadata::EMPTY];
	let mut socket = UdpSocket::new(
		stack,
		&mut rx_meta,
		&mut rx_buffer,
		&mut tx_meta,
		&mut tx_buffer,
	);

	// Server will send broadcasts to this port
	socket.bind(PORT).unwrap();

	// Sequence numbers are monotonically increasing. This is done to reject out-of-order packets
	// This along with serialization should maybe be done in Packets
	let mut tx_seq = 0;
	let mut rx_seq = 0;

	loop {
		// Either start sending or receive, if either is available
		let net = select(
			recv_bytes(&mut socket, &mut buffer),
			packets.serverbound.recv(),
		)
		.await;

		match (net, server_ip) {
			// There is inbound bytes that should be parsed and processed
			(Either::First((recv_len, endpoint)), _) => {
				// Try to optimistically parse all packets that come off the network
				let Ok(packet) = Packet::deserialize_from(&buffer[..recv_len]) else {
					trace!("Discarding {}", &buffer[..recv_len]);
					continue
				};
				let (seq, msg) = packet.split();

				// Cancel if sequence number is less than last seen. As of writing, SlimeVR server does not properly
				// count sequence numbers for clientbound packets, so it always sends 0. This still works, because we
				// only discard packets that were _less_ than previous
				if seq < rx_seq {
					warn!(
						"Out of order packet received: {}, we are at {} ({})",
						seq,
						rx_seq,
						defmt::Debug2Format(&msg)
					);
					continue;
				}

				// Hand the packet to rest of the system
				packets.clientbound.send(msg).await;
				rx_seq = seq;

				// If we received a valid packet, assume they are our real host
				if server_ip != Some(endpoint.addr) {
					info!(
						"Found SlimeVR server at {}, previously was {}",
						defmt::Debug2Format(&endpoint.addr),
						defmt::Debug2Format(&server_ip)
					);
					server_ip = Some(endpoint.addr);
				}
			}
			// There is pending outbound packet that should be sent
			(Either::Second(msg), Some(server_ip)) => {
				// Serialize the packet based on our send sequence number
				let Ok(len) = Packet::new(tx_seq, msg).serialize_into(&mut buffer) else {
					warn!("Failed to serialize outgoing packet");
					continue
				};
				tx_seq += 1;

				if let Err(e) = socket.send_to(&buffer[..len], (server_ip, PORT)).await
				{
					warn!("Failed to send #{}: {}", tx_seq, defmt::Debug2Format(&e));
				}
			}
			_ => (),
		}
	}
}

async fn connect_to_wifi(net: &mut NetConcrete) {
	let controller = &mut net.controller;
	let client_config = Configuration::Client(ClientConfiguration {
		ssid: super::SSID.into(),
		password: super::PASSWORD.into(),
		..Default::default()
	});
	controller.set_configuration(&client_config).unwrap();
	debug!("Starting wifi");
	controller.start().await.unwrap();
	debug!("Wifi started!");

	while let Err(e) = controller.connect().await {
		warn!("Failed to connect to wifi: {}", defmt::Debug2Format(&e));
		Timer::after(Duration::from_millis(4000)).await
	}
}

/// Asynchronously receive bytes from the network. This is a wrapper around UdpSocket::receive
/// Returns number of bytes read, receiving Ipv4 address and receiving port
async fn recv_bytes<'s>(
	socket: &mut UdpSocket<'s>,
	buffer: &mut [u8],
) -> (usize, IpEndpoint) {
	loop {
		match socket.recv_from(buffer).await {
			Ok(v) => return v,
			Err(UdpError::NoRoute) => warn!("UdpError::NoRoute"),
		}
	}
}

/// Drives the actual wifi stack
#[task]
async fn wifi_stack_task(stack: &'static crate::aliases::ඞ::NetStackConcrete) -> ! {
	stack.run().await
}
