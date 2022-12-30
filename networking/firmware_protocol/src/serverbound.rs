use alloc::format;
use deku::prelude::*;

use crate::{SlimeQuaternion, SlimeString};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
pub enum SbPacket {
	#[deku(id = "0")]
	Heartbeat,
	#[deku(id = "3")]
	Handshake {
		board: BoardType,
		// Handshake treats sensor type as i32, Sensor info as u8
		#[deku(pad_bytes_before = "3")]
		imu: ImuType,
		mcu: McuType,
		imu_info: (i32, i32, i32),
		build: i32,
		firmware: SlimeString,
		mac_address: [u8; 6],
	},
	#[deku(id = "4")]
	Acceleration {
		vector: (f32, f32, f32),
		sensor_id: u8,
	},
	#[deku(id = "10")]
	Ping { challenge: [u8; 4] },
	#[deku(id = "15")]
	SensorInfo {
		sensor_id: u8,
		sensor_status: SensorStatus,
		sensor_type: ImuType,
	},
	#[deku(id = "17")]
	RotationData {
		sensor_id: u8,
		data_type: SensorDataType,
		quat: SlimeQuaternion,
		calibration_info: u8,
	},
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u32", ctx = "_: deku::ctx::Endian", endian = "big")]
#[non_exhaustive]
/// The board design for a SlimeVR tracker
pub enum BoardType {
	#[deku(id = "1")]
	SlimeVRLegacy,
	#[deku(id = "2")]
	SlimeVRDev,
	#[deku(id = "3")]
	NodeMCU,
	#[deku(id = "4")]
	Custom,
	#[deku(id = "5")]
	WRoom32,
	#[deku(id = "6")]
	WemosD1Mini,
	#[deku(id = "7")]
	TTGOTBase,
	#[deku(id = "8")]
	ESP01,
	#[deku(id = "9")]
	SlimeVR,
	#[deku(id = "10")]
	LolinC3Mini,
	#[deku(id = "11")]
	Beetle32C3,
	#[deku(id = "12")]
	ESP32C3DevKitM1,
	#[deku(id_pat = "_")]
	Unknown(u32),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", ctx = "_: deku::ctx::Endian", endian = "big")]
#[non_exhaustive]
/// The intertial measurement unit in use
pub enum ImuType {
	#[deku(id = "1")]
	Mpu9250,
	#[deku(id = "2")]
	Mpu6500,
	#[deku(id = "3")]
	Bno080,
	#[deku(id = "4")]
	Bno085,
	#[deku(id = "5")]
	Bno055,
	#[deku(id = "6")]
	Mpu6050,
	#[deku(id = "7")]
	Bno086,
	#[deku(id = "8")]
	Bmi160,
	#[deku(id = "9")]
	Icm20948,
	#[deku(id_pat = "_")]
	Unknown(u8),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u32", ctx = "_: deku::ctx::Endian", endian = "big")]
#[non_exhaustive]
/// The family of the MCU in use
pub enum McuType {
	#[deku(id = "1")]
	Esp8266,
	#[deku(id = "2")]
	Esp32,
	#[deku(id_pat = "_")]
	Unknown(u32),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", ctx = "_: deku::ctx::Endian", endian = "big")]
/// Current status of the sensor
pub enum SensorStatus {
	#[deku(id = "0")]
	/// Sensor data is valid and up to date
	Ok,
	#[deku(id = "1")]
	/// Sensor is unavailable and data may be incorrect
	Offline,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", ctx = "_: deku::ctx::Endian", endian = "big")]
/// How should sensor data be treated
pub enum SensorDataType {
	#[deku(id = "1")]
	/// Sensor data is live and should be treated as-is
	Normal,
	#[deku(id = "2")]
	/// Never sent by C++ firmware
	Correction,
}

#[cfg(test)]
mod tests {
	use crate::*;

	// Compare data section of packet
	fn test(p: SbPacket, d: &[u8]) {
		let packet = Packet::new(0, p);
		let bytes = packet.to_bytes().unwrap();
		// Skip tag and seq
		assert_eq!(&bytes[4 + 8..], d);
		// Check deserialization
		assert_eq!(
			Packet::from_bytes((&bytes, 0)),
			Ok((([].as_slice(), 0), packet))
		);
	}

	#[test]
	fn heartbeat() {
		test(SbPacket::Heartbeat, &[]);
	}

	#[test]
	fn handshake() {
		test(
			SbPacket::Handshake {
				board: BoardType::SlimeVR,
				imu: ImuType::Bno085,
				mcu: McuType::Esp8266,
				imu_info: (1, 2, 3),
				build: 7,
				firmware: SlimeString::from("Test"),
				mac_address: *b"ferris",
			},
			&[
				0, 0, 0, 9, // Board
				0, 0, 0, // Pad
				4, // IMU
				0, 0, 0, 1, // MCU
				0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, // Info
				0, 0, 0, 7, // Build
				4, b'T', b'e', b's', b't', // Firmware
				b'f', b'e', b'r', b'r', b'i', b's', // MAC
			],
		);
	}

	#[test]
	fn acceleration() {
		test(
			SbPacket::Acceleration {
				vector: (
					f32::from_be_bytes([1, 2, 3, 4]),
					f32::from_be_bytes([5, 6, 7, 8]),
					f32::from_be_bytes([9, 10, 11, 12]),
				),
				sensor_id: 13,
			},
			&[
				1, 2, 3, 4, // X
				5, 6, 7, 8, // Y
				9, 10, 11, 12, // Z
				13, // ID
			],
		);
	}

	#[test]
	fn ping() {
		test(
			SbPacket::Ping {
				challenge: [1, 3, 3, 7],
			},
			&[1, 3, 3, 7],
		);
	}

	#[test]
	fn sensor_info() {
		test(
			SbPacket::SensorInfo {
				sensor_id: 40,
				sensor_status: SensorStatus::Offline,
				sensor_type: ImuType::Unknown(180),
			},
			&[
				40,  // ID
				1,   // Status
				180, // IMU
			],
		);
	}

	#[test]
	fn rotation_data() {
		test(
			SbPacket::RotationData {
				sensor_id: 40,
				data_type: SensorDataType::Normal,
				quat: SlimeQuaternion {
					i: f32::from_be_bytes([00, 01, 02, 03]),
					j: f32::from_be_bytes([10, 11, 12, 13]),
					k: f32::from_be_bytes([20, 21, 22, 23]),
					w: f32::from_be_bytes([30, 31, 32, 33]),
				},
				calibration_info: 127,
			},
			&[
				40, // ID
				1,  // Data type
				00, 01, 02, 03, // I
				10, 11, 12, 13, // J
				20, 21, 22, 23, // K
				30, 31, 32, 33, // W
				127, // Accuracy
			],
		);
	}
}
