#![no_std]

extern crate alloc;

#[cfg(test)]
mod test_deku;

pub use deku;

use alloc::format;
use alloc::string::FromUtf8Error;
use alloc::string::String;
use alloc::vec::Vec;

use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct SlimeQuaternion {
	pub i: f32,
	pub j: f32,
	pub k: f32,
	pub w: f32,
}

#[cfg(any(test, feature = "nalgebra031"))]
mod nalgebra031_impls {
	use super::*;
	use nalgebra031::Quaternion;

	impl From<Quaternion<f32>> for SlimeQuaternion {
		fn from(q: Quaternion<f32>) -> Self {
			Self {
				i: q.i as _,
				j: q.j as _,
				k: q.k as _,
				w: q.w as _,
			}
		}
	}
	impl From<SlimeQuaternion> for Quaternion<f32> {
		fn from(q: SlimeQuaternion) -> Self {
			Self::new(q.w as _, q.i as _, q.j as _, q.k as _)
		}
	}
}
#[cfg(any(test, feature = "nalgebra030"))]
mod nalgebra030_impls {
	use super::*;
	use nalgebra030::Quaternion;

	impl From<Quaternion<f32>> for SlimeQuaternion {
		fn from(q: Quaternion<f32>) -> Self {
			Self {
				i: q.i as _,
				j: q.j as _,
				k: q.k as _,
				w: q.w as _,
			}
		}
	}
	impl From<SlimeQuaternion> for Quaternion<f32> {
		fn from(q: SlimeQuaternion) -> Self {
			Self::new(q.w as _, q.i as _, q.j as _, q.k as _)
		}
	}
}

#[derive(PartialEq, Eq, Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct SlimeString {
	#[deku(update = "self.data.len()")]
	count: u8,
	#[deku(count = "count")]
	data: Vec<u8>,
}

impl From<&str> for SlimeString {
	fn from(s: &str) -> Self {
		let bytes = s.as_bytes();
		Self {
			count: bytes.len() as _,
			data: bytes.to_vec(),
		}
	}
}

impl From<String> for SlimeString {
	fn from(s: String) -> Self {
		let bytes = s.into_bytes();
		Self {
			count: bytes.len() as _,
			data: bytes,
		}
	}
}

impl SlimeString {
	#[allow(dead_code)]
	fn to_string(&self) -> Result<String, FromUtf8Error> {
		String::from_utf8(self.data.clone())
	}
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Packet {
	// TODO: This tag could really be dropped from the Rust side, but #[deku(temp)] is a bit wonky
	tag: u32,
	/// Sequence number for the packet. It is incremented for each subsequent packet and is used to reject out of order
	/// packets. This is sometimes referred to as the packet id
	seq: u64,
	#[deku(ctx = "*tag")]
	data: PacketData,
}

impl Packet {
	pub fn new(seq: u64, data: PacketData) -> Packet {
		Packet {
			tag: data.deku_id().unwrap(),
			seq,
			data,
		}
	}

	/// Serialize the packet into a byte slice, returning the number of bytes written. If the packet cannot fit into
	/// the buffer or data could not be serialied, Err is returned.
	pub fn serialize_into(&self, buf: &mut [u8]) -> Result<usize, ()> {
		// TODO: Deku should be extended to support in-place serialization instead of allocating here
		let bytes = self.to_bytes().map_err(|_| ())?;
		// Check we can fit the buffer
		if bytes.len() > buf.len() {
			return Err(());
		}
		buf[..bytes.len()].copy_from_slice(&bytes);
		Ok(bytes.len())
	}

	pub fn deserialize_from(buf: &[u8]) -> Result<Packet, ()> {
		match Packet::from_bytes((buf, 0)) {
			Ok(((tail, _tail_offset), packet)) if tail.len() == 0 => Ok(packet),
			_ => Err(()),
		}
	}

	pub fn split(self) -> (u64, PacketData) {
		(self.seq, self.data)
	}
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
pub enum PacketData {
	#[deku(id = "0")]
	Discovery,
	#[deku(id = "1")]
	Heartbeat,
	#[deku(id = "3")]
	Handshake {
		board: i32,
		imu: i32,
		mcu_type: i32,
		imu_info: (i32, i32, i32),
		build: i32,
		firmware: SlimeString,
		mac_address: [u8; 6],
	},
	#[deku(id = "4")]
	Acceleration {
		vector: (f32, f32, f32),
		sensor_id: Option<u8>,
	},
	#[deku(id = "10")]
	Ping { id: u32 },
	#[deku(id = "15")]
	SensorInfo {
		sensor_id: u8,
		sensor_status: u8,
		sensor_type: u8,
	},
	#[deku(id = "17")]
	RotationData {
		sensor_id: u8,
		data_type: u8,
		quat: SlimeQuaternion,
		calibration_info: u8,
	},
}
