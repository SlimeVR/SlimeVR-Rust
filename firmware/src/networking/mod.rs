use defmt::debug;
use embassy_executor::task;
use firmware_protocol::{BoardType, CbPacket, ImuType, McuType, SbPacket};

#[cfg(feature = "net-wifi")]
pub mod wifi;

mod packets;
pub use self::packets::{Connection, Packets};

#[cfg(feature = "net-wifi")]
pub use self::wifi::ඞ::network_task;
#[cfg(feature = "net-stubbed")]
pub use stubbed_network_task as network_task;

/// This does nothing, its a "fake" networking task meant to facilitate testing and
/// the initial port to a new platform (because there are no networking dependencies).
#[allow(dead_code)]
#[task]
pub async fn stubbed_network_task(packets: &'static Packets) -> ! {
	loop {
		// Dump network messages
		let _ = packets.serverbound.recv().await;
		defmt::trace!("pretending to do networking..");
	}
}

#[task]
pub async fn protocol_task(packets: &'static Packets) -> ! {
	debug!("protocol_task!");

	loop {
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

				*packets.connection.lock().await = Connection::Alive;
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
		}
	}
}
