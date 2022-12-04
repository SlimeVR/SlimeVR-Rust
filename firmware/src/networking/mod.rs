#[cfg(feature = "net-wifi")]
pub mod wifi;

pub async fn network_task() {
	#[cfg(feature = "net-wifi")]
	self::wifi::ඞ::network_task().await;
	#[cfg(feature = "net-stubbed")]
	stubbed_network_task().await;
}

/// This does nothing, its a "fake" networking task meant to facilitate testing and
/// the initial port to a new platform (because there are no networking dependencies).
#[allow(dead_code)]
pub async fn stubbed_network_task() {}
