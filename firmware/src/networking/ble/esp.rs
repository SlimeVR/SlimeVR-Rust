use crate::networking::Packets;
use bleps::{
	ad_structure::{
		create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED,
		LE_GENERAL_DISCOVERABLE,
	},
	Ble, Data, HciConnector,
};
use defmt::trace;
use embassy_futures::yield_now;
use esp_wifi::{self, ble::controller::BleConnector, current_millis};

pub async fn network_task(packets: &Packets) -> ! {
	// HCI is the host-controller interface, which lets the MCU communicate to the BLE hardware through a standard
	// command interface
	let connector = BleConnector {};
	let hci = HciConnector::new(connector, current_millis);
	let mut ble = Ble::new(&hci);

	ble.init().expect("Failed to initialize BLE hardware");

	// This will configure settings for the advertising server, like interval rate
	ble.cmd_set_le_advertising_parameters()
		.expect("Failed to configure advertising");

	ble.cmd_set_le_advertising_data(create_advertising_data(&[
		// Public advertising and and we do not support full Bluetooth connection (basic rate/extended data rate)
		AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
		AdStructure::CompleteLocalName("SlimeVR-Rust"),
		// Placeholder for SlimeVR advertised quaternion data. BLE advertising data size is tight, we should consider a
		// quantized quaternion format. 64 or even 32 bits should be enough
		// TODO: Unimplemented
		// AdStructure::ManufacturerSpecificData {
		// 	company_identifier: 0,
		// 	payload: &[0; 16],
		// },
	]))
	.expect("Failed to set advertising data");

	ble.cmd_set_le_advertise_enable(true)
		.expect("Failed to start advertising");

	loop {
		yield_now().await;
		let Some(event) = ble.poll() else { continue };

		trace!("BLE: {}", event);
	}
}
