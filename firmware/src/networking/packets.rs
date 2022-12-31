use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, mutex::Mutex};
use firmware_protocol::{CbPacket, SbPacket};

/// This struct represents the current server connection and is a mechanism for
/// shared state between network, protocol and imus
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Connection {
	Alive,
	Offline,
}

/// Packets is an accessor to internal logic <-> network messaging system
pub struct Packets {
	/// The latest `Message` that should be sent
	pub serverbound: Channel<NoopRawMutex, SbPacket, 1>,
	/// The latest `Message` that could be received
	pub clientbound: Channel<NoopRawMutex, CbPacket, 1>,
	/// Connection state
	// Kind of cringe and unbased, but works for now
	pub connection: Mutex<NoopRawMutex, Connection>,
}

impl Packets {
	pub const fn new() -> Packets {
		Packets {
			serverbound: Channel::new(),
			clientbound: Channel::new(),
			connection: Mutex::new(Connection::Offline),
		}
	}
}
