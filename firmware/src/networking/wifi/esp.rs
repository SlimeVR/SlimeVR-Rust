use defmt::{debug, warn};
use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};

use crate::aliases::ඞ::NetConcrete;

pub async fn connect_to_wifi(net: &mut NetConcrete) {
	let controller = &mut net.controller;
	let client_config = Configuration::Client(ClientConfiguration {
		ssid: super::SSID.into(),
		password: super::PASSWORD.into(),
		..Default::default()
	});
	controller.set_configuration(&client_config).unwrap();
	debug!("Starting wifi");
	controller.start().await.unwrap();
	debug!("Wifi started!");

	while let Err(e) = controller.connect().await {
		warn!("Failed to connect to wifi: {}", defmt::Debug2Format(&e));
		Timer::after(Duration::from_millis(4000)).await
	}
}
