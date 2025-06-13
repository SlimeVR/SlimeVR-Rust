use core::num::Wrapping;

use bleps::{
	ad_structure::{
		create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED,
		LE_GENERAL_DISCOVERABLE,
	},
	attribute_server::AttributeServer,
	Ble, HciConnector,
};
use defmt::{debug, error, warn};
use embassy_futures::yield_now;
use esp_wifi::{self, ble::controller::BleConnector, current_millis};
use firmware_protocol::{CbPacket, Packet, SbPacket};

use crate::aliases::ඞ::NetConcrete;
use crate::networking::Packets;

pub async fn network_task(packets: &Packets, _net: NetConcrete) -> ! {
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

	let mut sb_packet: Option<Packet<SbPacket>> = None;
	let mut cb_packet: Option<Packet<CbPacket>> = None;
	let mut sb_buf = [0; 512];

	let mut sb_callback = || {
		if let Some(packet) = sb_packet {
			let nbytes = packet.serialize_into(&mut sb_buf).unwrap();
			let data = &sb_buf[..nbytes];

			debug!("SEND: {=[u8]}", data);
			sb_packet = None;
			data
		} else {
			warn!("Data wasn't ready when we did work.");
			&[]
		}
	};
	let mut cb_callback = |offset: u16, data: &[u8]| {
		debug!("RECV: {} {=[u8]}", offset, data);

		if cb_packet.is_some() {
			panic!("This shouldn't have been possible!::<Packet> We are about to drop data.");
		}

		let Ok(packet) = Packet::deserialize_from(data) else {
			warn!("Discarding bogus packet");
			return;
		};
		cb_packet = Some(packet);
	};

	bleps_macros::gatt!([service {
		uuid: "133712e0-2354-11eb-9f10-fbc30a62cf38",
		characteristics: [characteristic {
			uuid: "13370000-2354-11eb-9f10-fbc30a62cf38",
			read: sb_callback,
			write: cb_callback,
		},],
	},]);

	let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes);

	let mut tx_seq = 0;
	let mut rx_seq = 0;
	debug!("Starting BLE loop");
	loop {
		// First we *must* handle any clientbound data, BEFORE doing work. Otherwise we may
		// experience data loss.
		if let Some(packet) = cb_packet {
			let (rx_seq_new, packet) = packet.split();
			if rx_seq_new <= rx_seq {
				warn!(
					"Out of order packet received: {}, we are at {} ({})",
					rx_seq_new,
					rx_seq,
					defmt::Debug2Format(&packet)
				);
			} else {
				packets.clientbound.send(packet).await;
				rx_seq = rx_seq_new;
			}
			cb_packet = None;
		}

		// Next, we load up the next packet.
		// TODO: Switch to debug assert or remove.
		assert!(sb_packet.is_none(), "This shouldn't be possible");
		sb_packet = Some(Packet::new(tx_seq, packets.serverbound.recv().await));
		tx_seq += 1;

		// Next, we will retrieve the next data to send the server.
		use bleps::attribute_server::WorkResult;
		match srv.do_work() {
			Ok(WorkResult::DidWork) => {}
			Ok(WorkResult::GotDisconnected) => warn!("BLE Disconnected!"),
			Err(err) => error!("Err: {}", defmt::Debug2Format(&err)),
		};
	}
}
