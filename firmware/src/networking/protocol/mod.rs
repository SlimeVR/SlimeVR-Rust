//! The protocol implementation to communicate with the SlimeVR Server.

mod packets;
pub use self::packets::Packets;

use defmt::debug;
use embassy_executor::task;
use embassy_futures::select::select;
use firmware_protocol::{
	BoardType, CbPacket, ImuType, McuType, SbPacket, SensorDataType, SensorStatus,
};

use crate::imu::Quat;
use crate::utils::Unreliable;

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
			do_work(packets, quat).await;
		}
	}
	.await
}

async fn do_work(packets: &Packets, quat: &Unreliable<Quat>) {
	let event = select(packets.clientbound.recv(), quat.wait()).await;
	use embassy_futures::select::Either;
	match event {
		Either::First(cb_packet) => todo!(),
		Either::Second(quat) => todo!(),
	}

	match packets.clientbound.recv().await {
		// Identify ourself when discovery packet is received
		CbPacket::Discovery => {
			packets
				.serverbound
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
			debug!("Handshake");

			// After handshake, we are supposed to send `SensorInfo` only once.
			packets
				.serverbound
				.send(SbPacket::SensorInfo {
					sensor_id: 0, // First sensor (of two)
					sensor_status: SensorStatus::Ok,
					sensor_type: ImuType::Unknown(0xFF),
				})
				.await;
			debug!("SensorInfo");
		}
		// When heartbeat is received, we should reply with heartbeat 0 aka Discovery
		// The protocol is asymmetric so its a bit unintuitive.
		CbPacket::Heartbeat => {
			packets.serverbound.send(SbPacket::Heartbeat).await;
		}
		// Pings are basically like heartbeats, just echo data back
		CbPacket::Ping { challenge } => {
			packets.serverbound.send(SbPacket::Ping { challenge }).await;
		}
		_ => (),
	}

	packets
		.serverbound
		.send(SbPacket::RotationData {
			sensor_id: 0,                      // First sensor
			data_type: SensorDataType::Normal, // Rotation data without magnetometer correction.
			quat: quat.wait().await.into_inner().into(),
			calibration_info: 0,
		})
		.await;
}
