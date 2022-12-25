//! An implementation of SlimeVR firmware, written in Rust.

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

#[cfg(bbq)]
mod bbq_logger;

use defmt::{debug, warn};
use embassy_executor::{task, Executor};
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::PacketType;
use imu::Quat;
use networking::Packets;
use static_cell::StaticCell;
use utils::Unreliable;

#[cfg(cortex_m)]
use cortex_m_rt::entry;
#[cfg(riscv)]
use riscv_rt::entry;
#[cfg(esp_xtensa)]
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
	#[cfg(bbq)]
	let bbq = defmt_bbq::init().unwrap();

	self::globals::setup();
	debug!("Booted");
	defmt::trace!("Trace");

	let p = self::peripherals::ඞ::get_peripherals();
	#[allow(unused)]
	let (bbq_peripheral, mut p) = p.bbq_peripheral();

	p.delay.delay_ms(500u32);
	debug!("Initialized peripherals");

	static PACKETS: StaticCell<Packets> = StaticCell::new();
	let packets: &'static Packets = PACKETS.init(Packets::new());

	static QUAT: StaticCell<Unreliable<Quat>> = StaticCell::new();
	let quat: &'static Unreliable<Quat> = QUAT.init(Unreliable::new());

	static EXECUTOR: StaticCell<Executor> = StaticCell::new();
	EXECUTOR.init(Executor::new()).run(move |s| {
		s.spawn(control_task(packets, quat)).unwrap();
		s.spawn(network_task(packets)).unwrap();
		s.spawn(imu_task(quat, p.i2c, p.delay)).unwrap();
		#[cfg(bbq)]
		s.spawn(logger_task(bbq, bbq_peripheral)).unwrap();
	});
}

#[task]
async fn control_task(packets: &'static Packets, quat: &'static Unreliable<Quat>) {
	debug!("Control task!");

	loop {
		match packets.clientbound.recv().await {
			// Identify ourself when discovery packet is received
			PacketType::Discovery => {
				packets
					.serverbound
					.send(PacketType::Handshake {
						board: 4,
						imu: 8,
						mcu_type: 2,
						imu_info: (1, 2, 3),
						build: 5,
						firmware: "SlimeVR-Rust".into(),
						mac_address: [0; 6],
					})
					.await;
			}
			// When heartbeat is received, we should reply with heartbeat 0 aka Discovery
			// The protocol is asymmetric so its a bit unintuitive. TODO: Split packet type into Server2Client and C2S
			PacketType::Heartbeat => {
				packets.serverbound.send(PacketType::Discovery).await;
			}
			packet => warn!("Unhandled packet {}", defmt::Debug2Format(&packet)),
		}

		packets
			.serverbound
			.send(PacketType::SensorInfo {
				sensor_id: 0,
				sensor_status: 1,
				sensor_type: 0,
			})
			.await;
		packets
			.serverbound
			.send(PacketType::RotationData {
				sensor_id: 0,
				data_type: 1,
				quat: quat.wait().await.into_inner().into(),
				calibration_info: 0,
			})
			.await;
	}
}

#[task]
async fn network_task(msg_signals: &'static Packets) {
	debug!("Network task!");
	crate::networking::network_task(msg_signals).await
}

#[task]
async fn imu_task(
	quat_signal: &'static Unreliable<Quat>,
	i2c: crate::aliases::ඞ::I2cConcrete<'static>,
	delay: crate::aliases::ඞ::DelayConcrete,
) {
	debug!("IMU Task!");
	crate::imu::imu_task(quat_signal, i2c, delay).await
}

#[cfg(bbq)]
#[task]
async fn logger_task(
	bbq: defmt_bbq::DefmtConsumer,
	logger_peripheral: crate::aliases::ඞ::BbqPeripheralConcrete<'static>,
) {
	crate::bbq_logger::ඞ::logger_task(bbq, logger_peripheral).await;
}
