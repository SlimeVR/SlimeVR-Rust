extern crate alloc;

use defmt::{error, info, trace, warn};
use embassy_futures::{
	select::{select, Either},
	yield_now,
};
use embedded_svc::ipv4::Interface;
// use esp_wifi::{
// 	create_network_stack_storage, current_millis, network_stack_storage,
// 	wifi::utils::create_network_interface,
// 	wifi_interface::{IoError as WifiError, Network, UdpSocket},
// };
// use smoltcp::{socket::UdpPacketMetadata, wire::Ipv4Address};

use crate::networking::protocol::Packets;
use firmware_protocol::Packet;

// SlimeVR default UDP port on both sides of connection
const PORT: u16 = 6969;

pub async fn network_task(packets: &Packets) -> ! {
	loop {
		yield_now().await;
	}

	// // TODO: Maybe we should look at the macros in the future for better config
	// // (socket_count, neighbour_cache_count, routes_store_count, multicast_store_count)
	// let mut storage = create_network_stack_storage!(3, 8, 1, 1);
	// let ethernet = create_network_interface(network_stack_storage!(storage));
	// let mut wifi = esp_wifi::wifi_interface::Wifi::new(ethernet);
	// super::connect_wifi(&mut wifi)
	// 	.await
	// 	.expect("Couldn't connect to wifi");
	//
	// let network = Network::new(wifi, current_millis);
	//
	// // Wait till DHCP assigns us an IP
	// let client_ip = loop {
	// 	yield_now().await;
	// 	network.work();
	// 	let Ok(ip) = network.get_ip_info() else { continue };
	// 	break ip.ip.octets();
	// };
	//
	// info!("DHCP IP: {}", client_ip);
	//
	// let mut server_ip = None; // We don't know the server ip yet.
	//
	// // Buffer size of 1536 matches modern MTU sizes and is more than enough for the SlimeVR protocol
	// // Unfortunately esp-wifi won't let us access the underlying tx/rx buffer. Unecessary copy here
	// let mut buffer = [0; 1536];
	// let mut rx_buffer = [0u8; 1536];
	// let mut tx_buffer = [0u8; 1536];
	// let mut rx_meta = [UdpPacketMetadata::EMPTY];
	// let mut tx_meta = [UdpPacketMetadata::EMPTY];
	// let mut socket = network.get_udp_socket(
	// 	&mut rx_meta,
	// 	&mut rx_buffer,
	// 	&mut tx_meta,
	// 	&mut tx_buffer,
	// );
	//
	// // Server will send broadcasts to this port
	// socket.bind(PORT).unwrap();
	//
	// // Sequence numbers are monotonically increasing. This is done to reject out-of-order packets
	// // This along with serialization should maybe be done in Packets
	// let mut tx_seq = 0;
	// let mut rx_seq = 0;
	//
	// // TODO: Implement with proper async select. So far there is no async counterpart of recv
	// loop {
	// 	// Either start sending or receive, if either is available
	// 	let net = select(
	// 		recv_bytes(&mut socket, &mut buffer),
	// 		packets.serverbound.recv(),
	// 	)
	// 	.await;
	//
	// 	match (net, server_ip) {
	// 		// There is inbound bytes that should be parsed and processed
	// 		(Either::First((len, addr, _port)), _) => {
	// 			// Try to optimistically parse all packets that come off the network
	// 			let Ok(packet) = Packet::deserialize_from(&buffer[..len]) else { trace!("Discarding {}", &buffer[..len]); continue };
	// 			let (seq, msg) = packet.split();
	//
	// 			// Cancel if sequence number is less than last seen. As of writing, SlimeVR server does not properly
	// 			// count sequence numbers for clientbound packets, so it always sends 0. This still works, because we
	// 			// only discard packets that were _less_ than previous
	// 			if seq < rx_seq {
	// 				warn!(
	// 					"Out of order packet received: {}, we are at {} ({})",
	// 					seq,
	// 					rx_seq,
	// 					defmt::Debug2Format(&msg)
	// 				);
	// 				continue;
	// 			}
	//
	// 			// Hand the packet to rest of the system
	// 			packets.clientbound.send(msg).await;
	// 			rx_seq = seq;
	//
	// 			// If we received a valid packet, assume they are our real host
	// 			if server_ip != Some(addr) {
	// 				info!(
	// 					"Found SlimeVR server at {}, previously was {}",
	// 					addr, server_ip
	// 				);
	// 				server_ip = Some(addr);
	// 			}
	// 		}
	// 		// There is pending outbound packet that should be sent
	// 		(Either::Second(msg), Some(server_ip)) => {
	// 			// Serialize the packet based on our send sequence number
	// 			let Ok(len) = Packet::new(tx_seq, msg).serialize_into(&mut buffer) else { warn!("Failed to serialize outgoing packet"); continue };
	// 			tx_seq += 1;
	//
	// 			if let Err(e) =
	// 				socket.send(Ipv4Address(server_ip), PORT, &buffer[..len])
	// 			{
	// 				warn!("Failed to send #{}: {}", tx_seq, defmt::Debug2Format(&e));
	// 			}
	// 		}
	// 		_ => (),
	// 	}
	// }
}

// /// Asynchronously receive bytes from the network. This is a wrapper around UdpSocket::receive
// /// Returns number of bytes read, receiving Ipv4 address and receiving port
// async fn recv_bytes<'s, 'n>(
// 	socket: &mut UdpSocket<'s, 'n>,
// 	buffer: &mut [u8],
// ) -> (usize, [u8; 4], u16) {
// 	loop {
// 		match socket.receive(buffer) {
// 			Ok(v) => return v,
// 			Err(WifiError::Other(smoltcp::Error::Exhausted)) => {}
// 			Err(WifiError::Other(e)) => error!("smoltcp error {}", e),
// 			Err(e) => error!("esp-wifi error {}", defmt::Debug2Format(&e)),
// 		}
// 		yield_now().await
// 	}
// }
