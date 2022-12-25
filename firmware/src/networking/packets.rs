use crate::utils::Reliable;
use firmware_protocol::{SBPacket, CBPacket};

/// Packets is an accessor to internal logic <-> network messaging system
pub struct Packets {
	/// The latest `Message` that should be sent
	pub serverbound: Reliable<SBPacket>,
	/// The latest `Message` that could be received
	pub clientbound: Reliable<CBPacket>,
}

impl Packets {
	pub const fn new() -> Packets {
		Packets {
			serverbound: Reliable::new(),
			clientbound: Reliable::new(),
		}
	}
}
