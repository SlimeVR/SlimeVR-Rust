//! A WIP sans-io implementation of the firmware protocol.
//!
//! sans-io means that it performs no io and can be used in async or non async code.

use crate::{CbPacket, SbPacket};

mod serialization;

// TODO: All of this code operates on deku stuff, can we make it generic?

pub enum ProtocolState {
	Disconnected(Disconnected),
	Connected(Connected),
}
impl ProtocolState {
	pub const fn new() -> Self {
		Self::Disconnected(Disconnected {})
	}
	pub fn received_msg(&mut self, m: CbPacket) -> Option<SbPacket> {
		// We are going to pass ownership of self, by stealing it out of the enum
		// temporarily, and putting it back in later.

		let mut taken = Self::new();
		core::mem::swap(self, &mut taken);
		let (mut taken, packet) = match taken {
			ProtocolState::Disconnected(s) => s.received_msg(m),
			ProtocolState::Connected(s) => s.received_msg(m),
		};
		core::mem::swap(self, &mut taken);
		packet
	}
}

pub struct Disconnected {}
impl Disconnected {
	pub fn received_msg(self, m: CbPacket) -> (ProtocolState, Option<SbPacket>) {
		todo!()
	}
}

pub struct Connected {}
impl Connected {
	pub fn received_msg(self, m: CbPacket) -> (ProtocolState, Option<SbPacket>) {
		todo!()
	}
	pub fn send_imu(&mut self, x: f32, y: f32, z: f32, w: f32) -> SbPacket {
		todo!()
	}
}
