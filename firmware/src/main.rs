#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]
// Needed to use `alloc` + `no_std`
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

mod aliases;
mod globals;
mod imu;
mod networking;
mod peripherals;
mod utils;

use defmt::debug;
use embassy_executor::{task, Executor};
use embedded_hal::blocking::delay::DelayMs;
use static_cell::StaticCell;

#[cfg(feature = "mcu-nrf52840")]
use cortex_m_rt::entry;
#[cfg(target_arch = "riscv32")]
use riscv_rt::entry;
#[cfg(esp_xtensa)]
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
	#[cfg(all(
		feature = "defmt-bbq",
		any(feature = "log-uart", feature = "log-usb-serial")
	))]
	let bbq = defmt_bbq::init().unwrap();

	self::globals::setup();
	debug!("Booted");
	defmt::trace!("Trace");

	let mut p = self::peripherals::ඞ::get_peripherals();
	p.delay.delay_ms(500u32);
	debug!("Initialized peripherals");

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |spawner| {
		spawner.spawn(network_task()).unwrap();
		spawner.spawn(imu_task(p.i2c, p.delay)).unwrap();
		#[cfg(all(feature = "defmt-bbq", feature = "log-uart"))]
		spawner.spawn(logger_task(bbq, p.uart)).unwrap();
		#[cfg(all(feature = "mcu-nrf52840", feature = "log-usb-serial"))]
		spawner.spawn(logger_task(bbq, p.usb_driver)).unwrap()
	});
}

#[task]
async fn network_task() {
	networking::network_task().await
}

#[task]
async fn imu_task(
	i2c: crate::aliases::ඞ::I2cConcrete<'static>,
	delay: crate::aliases::ඞ::DelayConcrete,
) {
	crate::imu::imu_task(i2c, delay).await
}

#[cfg(all(feature = "log-uart", feature = "mcu-nrf52840", feature = "defmt-bbq"))]
#[task]
async fn logger_task(
	mut bbq: defmt_bbq::DefmtConsumer,
	mut uart: crate::aliases::ඞ::UartConcrete<'static>,
) {
	use embassy_futures::yield_now;
	use embassy_nrf::uarte::Error;

	loop {
		let Ok(grant) = bbq.read() else {
			yield_now().await;
			continue;
		};
		let len = grant.buf().len();
		uart.write(b"got data: ").await;
		match uart.write_from_ram(grant.buf()).await {
			Err(Error::DMABufferNotInDataMemory) => {
				// unreachable!("bbq should always be in RAM")
				()
			}
			Err(Error::BufferZeroLength) | Err(Error::BufferTooLong) => (),
			Ok(()) => (),
			_ => (),
		};
		grant.release(len);
	}
}

#[cfg(all(feature = "log-usb-serial", feature = "mcu-nrf52840",))]
#[task]
async fn logger_task(
	mut bbq: defmt_bbq::DefmtConsumer,
	driver: crate::aliases::ඞ::UsbDriverConcrete<'static>,
) {
	use embassy_futures::{join::join, yield_now};

	debug!("Entered usb logger task");
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

	debug!("Created config");

	// Create embassy-usb DeviceBuilder using the driver and config.
	// It needs some buffers for building the descriptors.
	let mut device_descriptor = [0; 256];
	let mut config_descriptor = [0; 256];
	let mut bos_descriptor = [0; 256];
	let mut control_buf = [0; 64];

	let mut state = embassy_usb::class::cdc_acm::State::new();
	debug!("State");

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

	debug!("usb class");

	// Build the builder.
	let mut usb_device = builder.build();
	debug!("usb device");

	// Future for running the USB device.
	let usb_fut = usb_device.run();

	let write_fut = async {
		loop {
			usb_class.wait_connection().await;
			debug!("Awaited connection");
			let Ok(grant) = bbq.read() else {
				yield_now().await;
				continue;
			};
			let (result, len) = {
				let buf = grant.buf();
				let len = buf.len();

				(usb_class.write_packet(buf).await, len)
			};
			// let buf = b"hello";
			// TODO: Repeatedly write up to max packet size bytes.
			if let Err(_err) = result {
				// defmt::error!("{}", defmt::Debug2Format(&err));
				// There was an error, lets ignore it. Don't consume any of the buffer.
				grant.release(0);
			} else {
				grant.release(len);
				// defmt::debug!("Printed buf");
			}
		}
	};

	debug!("about to start future");

	// Run everything concurrently
	join(usb_fut, write_fut).await;
}
