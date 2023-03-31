use bleps::{
	ad_structure::{
		create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED,
		LE_GENERAL_DISCOVERABLE,
	},
	attribute_server::AttributeServer,
	Ble, HciConnector,
};
use defmt::{debug, error, trace, warn};
use embassy_futures::yield_now;
use esp_wifi::{self, ble::controller::BleConnector, current_millis};

use crate::aliases::ඞ::NetConcrete;
use crate::networking::Packets;

pub async fn network_task(_packets: &Packets, _net: NetConcrete) -> ! {
	// HCI is the host-controller interface, which lets the MCU communicate to the BLE hardware through a standard
	// command interface
	let connector = BleConnector {};
	let hci = HciConnector::new(connector, current_millis);
	let mut ble = Ble::new(&hci);

	ble.init().expect("Failed to initialize BLE hardware");
	debug!("Initialized BLE");

	// This will configure settings for the advertising server, like interval rate
	ble.cmd_set_le_advertising_parameters()
		.expect("Failed to configure advertising");

	ble.cmd_set_le_advertising_data(create_advertising_data(&[
		// Public advertising and and we do not support Bluetooth classic
		AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
		// If we specify the service uuids, it will show up in LightBlue as "1 service"
		// AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
		AdStructure::CompleteLocalName("SlimeVR-Rust"),
	]))
	.expect("Failed to set advertising data");

	ble.cmd_set_le_advertise_enable(true)
		.expect("Failed to start advertising");

	let mut rf = || &b"Hello Bare-Metal BLE"[..];
	let mut wf = |offset: u16, data: &[u8]| {
		trace!("RECEIVED: {} {=[u8]:a}", offset, data);
	};

	// let mut wf2 = |offset: u16, data: &[u8]| {
	// 	trace!("RECEIVED: {} {=[u8]:a}", offset, data);
	// };
	//
	// let mut rf3 = || &b"Hola!"[..];
	// let mut wf3 = |offset: u16, data: &[u8]| {
	// 	trace!("RECEIVED: Offset {}, data {=[u8]:a}", offset, data);
	// };

	use bleps_macros::gatt;

	gatt!([service {
		uuid: "133712e0-2354-11eb-9f10-fbc30a62cf38",
		characteristics: [
			characteristic {
				uuid: "13370000-2354-11eb-9f10-fbc30a62cf38",
				read: rf,
				write: wf,
			},
			// characteristic {
			// 	uuid: "13371111-2354-11eb-9f10-fbc30a62cf38",
			// 	write: wf2,
			// },
			// characteristic {
			// 	name: "my_characteristic",
			// 	uuid: "13372222-2354-11eb-9f10-fbc30a62cf38",
			// 	notify: true,
			// 	read: rf3,
			// 	write: wf3,
			// },
		],
	},]);

	let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes);

	debug!("Starting BLE loop");
	loop {
		yield_now().await;
		use bleps::attribute_server::WorkResult;
		match srv.do_work() {
			Ok(WorkResult::DidWork) => {}
			Ok(WorkResult::GotDisconnected) => warn!("BLE Disconnected!"),
			Err(err) => error!("Err: {}", defmt::Debug2Format(&err)),
		};
	}
}
