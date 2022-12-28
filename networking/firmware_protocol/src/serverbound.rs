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
		sensor_id: Option<u8>,
	},
	#[deku(id = "10")]
	Ping { id: u32 },
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
	// TODO: Figure out how to make Unknown(u32) with deku
	Unknown,
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
	// TODO: Figure out how to make Unknown(u8) with deku
	Unknown,
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
	// TODO: Figure out how to make Unknown(u32) with deku
	Unknown,
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
