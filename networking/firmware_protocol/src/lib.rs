#![no_std]

extern crate alloc;

#[cfg(test)]
mod test_deku;
mod serverbound;
mod clientbound;

use core::marker::PhantomData;

pub use deku;
use deku::ctx::Endian;
pub use serverbound::SBPacket;
pub use clientbound::CBPacket;

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
pub struct Packet<'a, D: DekuRead<'a, (Endian, u32)> + DekuWrite<(Endian, u32)>> {
	// TODO: This tag could really be dropped from the Rust side, but #[deku(temp)] is a bit wonky
	tag: u32,
	/// Sequence number for the packet. It is incremented for each subsequent packet and is used to reject out of order
	/// packets. This is sometimes referred to as the packet id
	seq: u64,
	#[deku(ctx = "*tag")]
	data: D,
	#[deku(skip)]
	_phantom: PhantomData<&'a ()>,
}

impl<'a, D: DekuRead<'a, (Endian, u32)> + DekuWrite<(Endian, u32)> + DekuEnumExt<'static, u32>> Packet<'a, D> {
	pub fn new(seq: u64, data: D) -> Self {
		Self {
			tag: data.deku_id().unwrap(),
			seq,
			data,
			_phantom: PhantomData
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

	pub fn deserialize_from(buf: &'a [u8]) -> Result<Self, ()> {
		match Packet::from_bytes((buf, 0)) {
			Ok(((tail, _tail_offset), packet)) if tail.len() == 0 => Ok(packet),
			_ => Err(()),
		}
	}

	pub fn split(self) -> (u64, D) {
		(self.seq, self.data)
	}
}
