#[cfg(feature = "net-wifi")]
pub mod wifi;

mod packets;
pub use self::packets::Packets;

pub async fn network_task(packets: &Packets) -> ! {
	#[cfg(feature = "net-wifi")]
	self::wifi::ඞ::network_task(packets).await;
	#[cfg(feature = "net-stubbed")]
	stubbed_network_task(packets).await
}

/// This does nothing, its a "fake" networking task meant to facilitate testing and
/// the initial port to a new platform (because there are no networking dependencies).
#[allow(dead_code)]
pub async fn stubbed_network_task(packets: &Packets) -> ! {
	loop {
		// Dump network messages
		let _ = packets.serverbound.recv().await;
		defmt::trace!("pretending to do networking..");
	}
}
