#![no_std]

extern crate alloc;

mod clientbound;
mod sansio;
mod serverbound;

pub use clientbound::*;
pub use deku;
use deku::ctx::Endian;
pub use serverbound::*;

use alloc::format;
use alloc::string::FromUtf8Error;
use alloc::string::String;
use alloc::vec::Vec;

use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "e", ctx = "e: deku::ctx::Endian")]
pub struct SlimeQuaternion {
	pub i: f32,
	pub j: f32,
	pub k: f32,
	pub w: f32,
}

#[allow(unused_macros)]
macro_rules! impl_Nalgebra {
	() => {
		use super::*;
		impl From<Quaternion<f32>> for SlimeQuaternion {
			fn from(q: Quaternion<f32>) -> Self {
				Self {
					i: q.i,
					j: q.j,
					k: q.k,
					w: q.w,
				}
			}
		}
		impl From<SlimeQuaternion> for Quaternion<f32> {
			fn from(q: SlimeQuaternion) -> Self {
				Self::new(q.w, q.i, q.j, q.k)
			}
		}
	};
}

#[cfg(any(test, feature = "nalgebra032"))]
mod nalgebra032_impls {
	use nalgebra032::Quaternion;
	impl_Nalgebra!();
}
#[cfg(any(test, feature = "nalgebra031"))]
mod nalgebra031_impls {
	use nalgebra031::Quaternion;
	impl_Nalgebra!();
}
#[cfg(any(test, feature = "nalgebra030"))]
mod nalgebra030_impls {
	use nalgebra030::Quaternion;
	impl_Nalgebra!();
}

#[derive(PartialEq, Eq, Debug, DekuRead, DekuWrite)]
#[deku(endian = "e", ctx = "e: deku::ctx::Endian")]
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

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct Packet<D>
where
	for<'a> D: DekuRead<'a, (Endian, u32)> + DekuWrite<(Endian, u32)>,
{
	/// Identifies the variant of the packet.
	tag: u32,
	/// Sequence number for the packet. It is incremented for each subsequent packet and is used to reject out of order
	/// packets. This is sometimes referred to as the packet id
	seq: u64,
	#[deku(ctx = "*tag")]
	data: D,
}

impl<D> Packet<D>
where
	for<'a> D: DekuRead<'a, (Endian, u32)>
		+ DekuWrite<(Endian, u32)>
		+ DekuEnumExt<'static, u32>,
{
	pub fn new(seq: u64, data: D) -> Self {
		Self {
			tag: data.deku_id().unwrap(),
			seq,
			data,
		}
	}

	/// Serialize the packet into a byte slice, returning the number of bytes written. If the packet cannot fit into
	/// the buffer or data could not be serialied, Err is returned.
	pub fn serialize_into(&self, buf: &mut [u8]) -> Result<usize, SerializeError> {
		// TODO: Deku should be extended to support in-place serialization instead of allocating here
		let bytes = self.to_bytes()?;
		// Check we can fit the buffer
		if bytes.len() > buf.len() {
			return Err(SerializeError::BufferTooSmall);
		}
		buf[..bytes.len()].copy_from_slice(&bytes);
		Ok(bytes.len())
	}

	pub fn deserialize_from(buf: &[u8]) -> Result<Self, DeserializeError> {
		match Packet::from_bytes((buf, 0)) {
			Ok(((tail, _tail_offset), packet)) => {
				if tail.is_empty() {
					Ok(packet)
				} else {
					Err(DeserializeError::BytesRemaining)
				}
			}
			Err(deku) => Err(DeserializeError::Deku(deku)),
		}
	}

	/// Returns a tuple of the sequence number, and the `PacketData`.
	pub fn split(self) -> (u64, D) {
		(self.seq, self.data)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeError {
	Deku(::deku::DekuError),
	BufferTooSmall,
}
impl From<::deku::DekuError> for SerializeError {
	fn from(deku: ::deku::DekuError) -> Self {
		Self::Deku(deku)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeError {
	Deku(::deku::DekuError),
	/// Unexpectedly had bytes remaining after deserialization.
	BytesRemaining,
}
impl From<::deku::DekuError> for DeserializeError {
	fn from(deku: ::deku::DekuError) -> Self {
		Self::Deku(deku)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Dummy packet data used for testing
	#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
	#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
	enum Dummy {
		#[deku(id = "0")]
		D0,
		#[deku(id = "1")]
		D1,
		#[deku(id = "2")]
		D2 { val: u32 },
	}

	#[test]
	fn packet_d0() {
		// Generate 10 packets with increasing sequence number and check round trip se/deserialization
		for i in 0..10 {
			let packet = Packet::new(i, Dummy::D0);
			let bytes = packet.to_bytes().unwrap();
			#[rustfmt::skip]
			let expected = [
				0, 0, 0, 0, // Variant
				0, 0, 0, 0, 0, 0, 0, i as u8, // Sequence
				// Data
			];
			assert_eq!(bytes, expected);
			assert_eq!(
				Packet::from_bytes((&bytes, 0)),
				Ok((([].as_slice(), 0), packet))
			);
		}
	}

	#[test]
	fn packet_d1() {
		// Generate 10 packets with increasing sequence number and check round trip se/deserialization
		for i in 0..10 {
			let packet = Packet::new(i, Dummy::D1);
			let bytes = packet.to_bytes().unwrap();
			#[rustfmt::skip]
			let expected = [
				0, 0, 0, 1, //
				0, 0, 0, 0, 0, 0, 0, i as u8, // Sequence
				 // Data
			];
			assert_eq!(bytes, expected);
			assert_eq!(
				Packet::from_bytes((&bytes, 0)),
				Ok((([].as_slice(), 0), packet))
			);
		}
	}

	#[test]
	fn packet_d2() {
		// Generate 10 packets with increasing sequence number and check round trip se/deserialization
		for i in 0..10 {
			let packet = Packet::new(i, Dummy::D2 { val: i as u32 + 20 });
			let bytes = packet.to_bytes().unwrap();
			#[rustfmt::skip]
			let expected = [
				0, 0, 0, 2, // Variant
				0, 0, 0, 0, 0, 0, 0, i as u8, // Sequence
				0, 0, 0, i as u8 + 20 // Data
			];
			assert_eq!(bytes, expected);
			assert_eq!(
				Packet::from_bytes((&bytes, 0)),
				Ok((([].as_slice(), 0), packet))
			);
		}
	}
}
