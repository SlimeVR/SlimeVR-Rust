pub mod protocol;
#[cfg(feature = "net-wifi")]
pub mod wifi;

#[cfg(feature = "net-ble")]
pub mod ble;

use defmt::debug;
use embassy_executor::{task, Spawner};

use crate::aliases::ඞ::NetConcrete;
use crate::networking::protocol::Packets;

// TODO: Does this need a larger task bool because `network_task` may spawn a task?
#[task]
pub async fn network_task(
	#[allow(unused_variables)] spawner: Spawner,
	msg_signals: &'static Packets,
	net: NetConcrete,
) {
	debug!("Network task");
	#[cfg(feature = "net-wifi")]
	self::wifi::network_task(spawner, msg_signals, net).await;
	#[cfg(feature = "net-ble")]
	self::ble::ඞ::network_task(msg_signals, net).await;
	#[cfg(feature = "net-stubbed")]
	stubbed_network_task(msg_signals, net).await;
}

/// This does nothing, its a "fake" networking task meant to facilitate testing and
/// the initial port to a new platform (because there are no networking dependencies).
#[allow(dead_code)]
async fn stubbed_network_task(packets: &Packets, _net: NetConcrete) -> ! {
	loop {
		// Dump network messages
		let _ = packets.serverbound.recv().await;
	}
}
