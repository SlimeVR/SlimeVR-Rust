//! The protocol implementation to communicate with the SlimeVR Server.

mod packets;
pub use self::packets::Packets;

use defmt::{debug, trace};
use embassy_executor::task;
use embassy_futures::select::{select, Either};

use firmware_protocol::{
	BoardType, CbPacket, ImuType, McuType, SbPacket, SensorDataType, SensorStatus,
};

use crate::imu::Quat;
use crate::utils::{Reliable, Unreliable};

#[allow(dead_code)]
mod v2;

#[task]
pub async fn control_task(
	packets: &'static Packets,
	quat: &'static Unreliable<Quat>,
) -> ! {
	debug!("Control task!");
	async {
		loop {
			match select(packets.clientbound.recv(), quat.wait()).await {
				Either::First(cb_msg) => {
					handle_cb_msg(cb_msg, &packets.serverbound).await
				}
				Either::Second(quat_msg) => {
					handle_quat(quat_msg, &packets.serverbound).await
				}
			}
		}
	}
	.await
}

async fn handle_cb_msg(cb_msg: CbPacket, sb_chan: &Reliable<SbPacket>) {
	match cb_msg {
		// Identify ourself when discovery packet is received
		CbPacket::Discovery => {
			trace!("protocol: received Discovery");
			sb_chan
				.send(SbPacket::Handshake {
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
				})
				.await;

			// After handshake, we are supposed to send `SensorInfo` only once.
			sb_chan
				.send(SbPacket::SensorInfo {
					sensor_id: 0, // First sensor (of two)
					sensor_status: SensorStatus::Ok,
					sensor_type: ImuType::Unknown(0xFF),
				})
				.await;
		}
		// When heartbeat is received, we should reply with heartbeat 0 aka Discovery
		// The protocol is asymmetric so its a bit unintuitive.
		CbPacket::Heartbeat => {
			trace!("protocol: received Heartbeat");
			sb_chan.send(SbPacket::Heartbeat).await;
		}
		// Pings are basically like heartbeats, just echo data back
		CbPacket::Ping { challenge } => {
			trace!("protocol: received Ping");
			sb_chan.send(SbPacket::Ping { challenge }).await;
		}
		_ => (),
	}
}

async fn handle_quat(quat: Quat, sb_chan: &Reliable<SbPacket>) {
	sb_chan
		.send(SbPacket::RotationData {
			sensor_id: 0,                      // First sensor
			data_type: SensorDataType::Normal, // Rotation data without magnetometer correction.
			quat: quat.into_inner().into(),
			calibration_info: 0,
		})
		.await
}
