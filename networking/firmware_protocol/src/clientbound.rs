use alloc::format;
use deku::prelude::*;

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
pub enum CbPacket {
	#[deku(id = "0")]
	Discovery,
	#[deku(id = "1")]
	Heartbeat,
	#[deku(id = "10")]
	Ping {
		/// Arbitrary bytes sent by the server that must be echoed
		challenge: [u8; 4],
	},
}

#[cfg(test)]
mod tests {
	use crate::*;

	// Compare data section of packet
	fn test(p: CbPacket, d: &[u8]) {
		let bytes = Packet::new(0, p).to_bytes().unwrap();
		// Skip tag and seq
		assert_eq!(&bytes[4 + 8..], d);
	}

	#[test]
	fn discovery() {
		test(CbPacket::Discovery, &[]);
	}

	#[test]
	fn heartbeat() {
		test(CbPacket::Discovery, &[]);
	}

	#[test]
	fn ping() {
		test(
			CbPacket::Ping {
				challenge: [1, 2, 3, 4],
			},
			&[1, 2, 3, 4],
		);
	}
}
