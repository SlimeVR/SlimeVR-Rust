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
	/// u32::from_be_bytes([3, b'H', b'e', b'y']) -> 55076217
	#[deku(id = "55076217")]
	HandshakeResponse {
		/// Char. SlimeVR Server sends '5' = 53
		version: u8,
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

	#[test]
	fn handshake_response() {
		// 3"Hey" -> [3, 72, 101, 121] -> 55076217
		// " OVR =D " -> [32, 79, 86, 82, 32, 61, 68, 32]
		// u64::from_be_bytes([32, 79, 86, 82, 32, 61, 68, 32]) -> 2328174443102028832
		let packet = Packet::new(
			2_328_174_443_102_028_832,
			CbPacket::HandshakeResponse { version: b'5' },
		)
		.to_bytes()
		.unwrap();

		assert_eq!(&packet, &"\x03Hey OVR =D 5".as_bytes());
	}
}
