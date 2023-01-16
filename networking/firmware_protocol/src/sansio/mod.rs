//! A WIP sans-io implementation of the firmware protocol.
//!
//! sans-io means that it performs no io and can be used in async or non async code.
//!
//! TODO: All of this code operates on deku stuff, can we make it generic?

use crate::{
	BoardType, CbPacket, ImuType, McuType, SbPacket, SensorDataType, SensorStatus,
	SlimeQuaternion,
};

use replace_with::replace_with;

/// A buffer of `SbPacket` that *must all be sent* before other functions on the
/// protocol are invoked.
#[derive(Debug)]
pub struct SbBuf(heapless::Deque<SbPacket, 2>);
impl SbBuf {
	pub const fn new() -> Self {
		Self(heapless::Deque::new())
	}

	pub fn peek(&self) -> Option<&SbPacket> {
		self.0.front()
	}

	pub fn pop(&mut self) -> Option<SbPacket> {
		self.0.pop_front()
	}

	pub const fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	fn push(&mut self, sb: SbPacket) {
		self.0.push_back(sb).expect(
			"Unexpectedly ran out of space in deque, this should have been impossible.",
		)
	}
}

/// Panics with a standardized message if the `SbBuf` is empty.
macro_rules! assert_empty {
	($sb_buf:expr) => {
		assert!(!$sb_buf.is_empty(), "`SbBuf` must be empty!");
	};
}

#[derive(Debug, derive_more::From)]
pub enum State {
	Disconnected(Disconnected),
	Connected(Connected),
}
impl State {
	pub const fn new() -> Self {
		Self::Disconnected(Disconnected)
	}

	/// Process a newly received message, enqueueing any serverbound messages in
	/// `sb_buf`.
	///
	/// # Panics
	/// Panics if `!sb_buf.is_empty()`. You are only supposed to call this function when
	/// all serverbound packets have already been sent.
	pub fn received_msg<'b>(&mut self, cb: CbPacket, sb_buf: &'b mut SbBuf) {
		assert_empty!(sb_buf);
		// This allows us to temporarily steal from `self` even though we don't have
		// ownership.
		replace_with(
			self,
			// TODO: This will be a double panic and halt, making it hard to debug if we
			// ever actually hit this. Is there a better way? Do we even care?
			|| unreachable!("Panic should be impossible because we already asserted"),
			|taken| {
				// Now we can use the owned value in functions that expect an owned type.
				match taken {
					State::Disconnected(s) => s.received_msg(cb, sb_buf),
					State::Connected(s) => s.received_msg(cb, sb_buf),
				}
			},
		);
	}
}

#[derive(Debug)]
pub struct Disconnected;
impl Disconnected {
	/// Process a newly received message, enqueueing any serverbound messages in
	/// `sb_buf`.
	///
	/// # Panics
	/// Panics if `!sb_buf.is_empty()`. You are only supposed to call this function when
	/// all serverbound packets have already been sent.
	pub fn received_msg<'b>(self, cb: CbPacket, sb_buf: &'b mut SbBuf) -> State {
		assert_empty!(sb_buf);
		match cb {
			CbPacket::Discovery => {
				sb_buf.push(SbPacket::Handshake {
					// TODO: Compile time constants for board and MCU
					board: BoardType::Custom,
					// Should this IMU type be whatever the first IMU of the system is?
					imu: ImuType::Unknown(0xFF),
					mcu: McuType::Esp32,
					imu_info: (0, 0, 0), // These appear to be inert
					// Needs to be >=9 to use newer protocol, this is hard-coded in
					// the java server :(
					build: 10,
					firmware: "SlimeVR-Rust".into(),
					mac_address: [0; 6],
				});

				sb_buf.push(SbPacket::SensorInfo {
					sensor_id: 0, // First sensor (of two)
					sensor_status: SensorStatus::Ok,
					sensor_type: ImuType::Unknown(0xFF),
				});

				self.into()
			}
			_ => self.into(), // Don't care about other packet types.
		}
	}
}

#[derive(Debug)]
pub struct Connected {}
impl Connected {
	/// Process a newly received message, enqueueing any serverbound messages in
	/// `sb_buf`.
	///
	/// # Panics
	/// Panics if `!sb_buf.is_empty()`. You are only supposed to call this function when
	/// all serverbound packets have already been sent.
	pub fn received_msg<'b>(self, cb: CbPacket, sb_buf: &'b mut SbBuf) -> State {
		assert_empty!(sb_buf);
		match cb {
			// When heartbeat is received, we should reply with another heartbeat.
			CbPacket::Heartbeat => {
				sb_buf.push(SbPacket::Heartbeat);
				self.into()
			}
			// Pings are basically like heartbeats, but they also echo data back
			CbPacket::Ping { challenge } => {
				sb_buf.push(SbPacket::Ping { challenge });
				self.into()
			}
			_ => self.into(), // Don't care about other packet types.
		}
	}

	/// Enqueues an imu update in `SbBuf`.
	///
	/// # Panics
	/// Panics if `!sb_buf.is_empty()`. You are only supposed to call this function when
	/// all serverbound packets have already been sent.
	pub fn send_imu(
		self,
		sensor_id: u8,
		quat: SlimeQuaternion,
		sb_buf: &mut SbBuf,
	) -> State {
		assert_empty!(sb_buf);
		sb_buf.push(SbPacket::RotationData {
			sensor_id,
			data_type: SensorDataType::Normal,
			quat,
			calibration_info: 0,
		});
		self.into()
	}
}
