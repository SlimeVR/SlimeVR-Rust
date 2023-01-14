pub mod protocol;
#[cfg(feature = "net-wifi")]
pub mod wifi;

#[cfg(feature = "net-ble")]
pub mod ble;

use defmt::debug;
use embassy_executor::task;

use crate::networking::protocol::Packets;

#[task]
pub async fn network_task(msg_signals: &'static Packets) {
	debug!("Network task");
	#[cfg(feature = "net-wifi")]
	self::wifi::ඞ::network_task(msg_signals).await;
	#[cfg(feature = "net-ble")]
	self::ble::ඞ::network_task(msg_signals).await;
	#[cfg(feature = "net-stubbed")]
	stubbed_network_task(msg_signals).await;
}

/// This does nothing, its a "fake" networking task meant to facilitate testing and
/// the initial port to a new platform (because there are no networking dependencies).
#[allow(dead_code)]
async fn stubbed_network_task(packets: &Packets) -> ! {
	loop {
		// Dump network messages
		let _ = packets.serverbound.recv().await;
		defmt::trace!("pretending to do networking..");
	}
}
