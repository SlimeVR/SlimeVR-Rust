//! THIS MODULE IS WIP AND NOT FINISHED YET
//!
//! Manages the slimevr firmware protocol. All (de)serialization and state associated with
//! the protocol happens in here.
#![allow(dead_code)]

mod serialize;

use crate::{Quat, Unreliable};

use embassy_sync::{
	blocking_mutex::raw::NoopRawMutex,
	channel::{Channel, Receiver, Sender},
};
use firmware_protocol::{CbPacket, SbPacket};

pub struct ImuHandle {
	signal: &'static Unreliable<Quat>,
}
impl ImuHandle {
	/// Send the latest quaternion. May drop the data.
	pub fn send_quat(&mut self, q: Quat) {
		self.signal.signal(q)
	}
}

pub struct Protocol {
	quat_signal: &'static Unreliable<Quat>,
	sb: Channel<NoopRawMutex, SbPacket, 1>, // TODO: Should these actually be channels?
	cb: Channel<NoopRawMutex, CbPacket, 1>,
}
impl Protocol {
	pub fn new(quat_signal: &'static Unreliable<Quat>) -> (Protocol, ImuHandle) {
		let self_ = Self {
			quat_signal,
			sb: Channel::new(),
			cb: Channel::new(),
		};
		(
			self_,
			ImuHandle {
				signal: quat_signal,
			},
		)
	}

	/// Gets a `Receiver` for serverbound packets that ought to be sent.
	pub fn serverbound(&self) -> Receiver<'_, NoopRawMutex, SbPacket, 1> {
		self.sb.receiver()
	}

	/// Gets a `Sender` for clientbound packets.
	pub fn clientbound(&self) -> Sender<'_, NoopRawMutex, CbPacket, 1> {
		self.cb.sender()
	}

	/// Do networking! Makes progress on producing and consuming packets. This should be
	/// called repeatedly by the network task to ensure progress is made.
	pub async fn work() {
		todo!()
	}

	/// Resets the protocol to its initial state, for example when network connection
	/// failed.
	pub async fn reset(&mut self) {
		todo!()
	}
}
