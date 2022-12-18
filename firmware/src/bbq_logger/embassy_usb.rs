//! defmt-bbq logger using usb serial + embassy

use defmt::debug;
use defmt_bbq::DefmtConsumer;
use embassy_futures::{join::join, yield_now};
use embassy_usb::driver::EndpointError;

pub async fn logger_task(
	mut bbq: DefmtConsumer,
	driver: crate::aliases::ඞ::UsbDriverConcrete<'static>,
) {
	debug!("USB logger task!");
	// Code based on: https://github.com/embassy-rs/embassy/blob/ebc735008f0b1725b22c944cc5f95fe1aacc665b/examples/nrf/src/bin/usb_serial.rs#L31-L72

	// Create embassy-usb Config
	let config = {
		let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
		config.manufacturer = Some("Embassy");
		config.product = Some("USB-serial example");
		config.serial_number = Some("12345678");
		config.max_power = 100;
		config.max_packet_size_0 = 64;

		// Required for windows compatiblity.
		// https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
		config.device_class = 0xEF;
		config.device_sub_class = 0x02;
		config.device_protocol = 0x01;
		config.composite_with_iads = true;

		config
	};

	// Create embassy-usb DeviceBuilder using the driver and config.
	// It needs some buffers for building the descriptors.
	let mut device_descriptor = [0; 256];
	let mut config_descriptor = [0; 256];
	let mut bos_descriptor = [0; 256];
	let mut control_buf = [0; 64];

	let mut state = embassy_usb::class::cdc_acm::State::new();

	let mut builder = embassy_usb::Builder::new(
		driver,
		config,
		&mut device_descriptor,
		&mut config_descriptor,
		&mut bos_descriptor,
		&mut control_buf,
		None,
	);

	// Create classes on the builder.
	let mut usb_class =
		embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, &mut state, 64);

	// Build the builder.
	let mut usb_device = builder.build();

	// Future for running the USB device.
	let usb_fut = usb_device.run();

	let write_fut = async {
		usb_class.wait_connection().await;
		debug!("Awaited connection for first time");
		loop {
			let Ok(grant) = bbq.read() else {
				yield_now().await;
				continue;
			};
			let (result, len) = {
				let buf = grant.buf();
				let len = buf.len();

				(usb_class.write_packet(buf).await, len)
			};
			// TODO: Repeatedly write up to max packet size bytes.
			match result {
				Err(EndpointError::BufferOverflow) => {
					// There was an error, lets ignore it. Don't consume any of the buffer.
					grant.release(0);
				}
				Err(EndpointError::Disabled) => {
					// There was an error, lets ignore it. Don't consume any of the buffer.
					grant.release(0);
					usb_class.wait_connection().await;
					debug!("Awaited connection");
				}
				Ok(()) => grant.release(len),
			}
		}
	};

	// Run everything concurrently
	join(usb_fut, write_fut).await;
}
