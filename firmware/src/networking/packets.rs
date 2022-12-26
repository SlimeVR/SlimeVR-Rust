use crate::utils::Reliable;
use firmware_protocol::PacketData;

/// Packets is an accessor to internal logic <-> network messaging system
pub struct Packets {
	/// The latest `Message` that should be sent
	pub serverbound: Reliable<PacketData>,
	/// The latest `Message` that could be received
	pub clientbound: Reliable<PacketData>,
}

impl Packets {
	pub const fn new() -> Packets {
		Packets {
			serverbound: Reliable::new(),
			clientbound: Reliable::new(),
		}
	}
}
