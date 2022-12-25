use alloc::format;
use deku::prelude::*;

use crate::{SlimeString, SlimeQuaternion};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
pub enum SBPacket {
	#[deku(id = "0")]
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
