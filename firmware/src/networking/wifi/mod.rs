#[cfg(mcu_f_esp32)]
#[path = "esp.rs"]
pub mod ඞ;

use defmt::{debug, info, trace, warn};
use embassy_executor::{task, Spawner};
use embassy_futures::select::{select, Either};
use embassy_net::udp::{Error as UdpError, UdpSocket};
use embassy_time::{Duration, Timer};
use firmware_protocol::{Packet, SbPacket};
use smoltcp::socket::udp::PacketMetadata as UdpPacketMetadata;
use smoltcp::wire::{IpAddress, IpEndpoint};

use crate::aliases::ඞ::NetConcrete;
use crate::networking::protocol::Packets;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

// SlimeVR default UDP port on both sides of connection
const PORT: u16 = 6969;

// Sequence numbers are monotonically increasing.
// This is done to reject out-of-order packets.
// This along with serialization should maybe be done in Packets.
#[derive(PartialOrd, Eq, PartialEq, Debug, Copy, Clone, defmt::Format)]
struct RxSeq(u64);

#[derive(PartialOrd, Eq, PartialEq, Debug, Copy, Clone, defmt::Format)]
struct TxSeq(u64);

struct State {
	packets: &'static Packets,
	rx_seq: RxSeq,
	tx_seq: TxSeq,
	server_ip: Option<IpAddress>,
}
impl State {
	fn new(packets: &'static Packets) -> Self {
		State {
			packets,
			rx_seq: RxSeq(0),
			tx_seq: TxSeq(0),
			server_ip: None,
		}
	}

	// If the packet was valid and not out of order, returns its Rx sequence number
	async fn on_recv(&mut self, data: &[u8], endpoint: IpEndpoint) {
		// Try to optimistically parse all packets that come off the network
		let Ok(packet) = Packet::deserialize_from(data) else {
			trace!("Discarding {}", data);
			return;
		};

		let (new_seq, msg) = packet.split();
		let new_seq = RxSeq(new_seq);

		// Cancel if sequence number is less than last seen. As of writing, SlimeVR server does not properly
		// count sequence numbers for clientbound packets, so it always sends 0. This still works, because we
		// only discard packets that were _less_ than previous
		if new_seq < self.rx_seq {
			warn!(
				"Out of order packet received: {}, we are at {} ({})",
				new_seq,
				self.rx_seq,
				defmt::Debug2Format(&msg)
			);
			return;
		}
		self.rx_seq = new_seq;

		// If we received a valid packet, assume they are our real host
		if self.server_ip != Some(endpoint.addr) {
			info!(
				"Found SlimeVR server at {}, previously was {}",
				defmt::Debug2Format(&endpoint.addr),
				defmt::Debug2Format(&self.server_ip)
			);
			self.server_ip = Some(endpoint.addr);
		}

		// Hand the packet to rest of the system
		self.packets.clientbound.send(msg).await;
	}

	/// Returns num bytes of outbound buffer
	fn on_send(&mut self, outbound: SbPacket, out_buf: &mut [u8]) -> Option<usize> {
		// Serialize the packet based on our send sequence number
		let Ok(len) = Packet::new(self.tx_seq().0, outbound).serialize_into(out_buf) else {
			warn!("Failed to serialize outgoing packet");
			return None;
		};
		self.tx_seq.0 += 1;

		Some(len)
	}

	fn tx_seq(&self) -> TxSeq {
		self.tx_seq
	}

	fn server_ip(&self) -> Option<IpAddress> {
		self.server_ip
	}
}

/// Drives the actual wifi stack
#[task]
async fn wifi_stack_task(stack: &'static crate::aliases::ඞ::NetStackConcrete) -> ! {
	debug!("Wifi stack task!");
	stack.run().await
}

pub async fn network_task(
	spawner: Spawner,
	packets: &'static Packets,
	mut net: NetConcrete,
) -> ! {
	spawner.spawn(wifi_stack_task(net.stack)).unwrap();
	self::ඞ::connect_to_wifi(&mut net).await;

	let stack = net.stack;

	let ip = loop {
		if let Some(ip) = stack.config().map(|c| c.address) {
			break ip;
		}
		Timer::after(Duration::from_millis(500)).await;
	};
	debug!("Got ip: {}", defmt::Debug2Format(&ip));

	let mut state = State::new(packets);

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

	loop {
		// Either start sending or receive, if either is available
		let net = select(
			recv_bytes(&mut socket, &mut buffer),
			packets.serverbound.recv(),
		)
		.await;

		match (net, state.server_ip()) {
			// There is inbound bytes that should be parsed and processed
			(Either::First((recv_len, endpoint)), _) => {
				state.on_recv(&buffer[..recv_len], endpoint).await
			}
			// There is pending outbound packet that should be sent
			(Either::Second(msg), Some(server_ip)) => {
				let Some(nbytes) = state.on_send(msg, &mut buffer) else {
					continue;
				};

				if let Err(e) =
					socket.send_to(&buffer[..nbytes], (server_ip, PORT)).await
				{
					warn!(
						"Failed to send #{}: {}",
						state.tx_seq(),
						defmt::Debug2Format(&e)
					);
				}
			}
			_ => (),
		}
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
