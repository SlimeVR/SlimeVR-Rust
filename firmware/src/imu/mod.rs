mod stubbed;

#[cfg(feature = "imu-stubbed")]
mod ඞ {
	pub use crate::imu::stubbed::*;
}

#[cfg(feature = "imu-mpu6050")]
#[path = "mpu6050.rs"]
mod ඞ;

#[cfg(feature = "imu-bmi160")]
#[path = "bmi160/mod.rs"]
mod ඞ;

use crate::{networking::Packets, utils::nb2a};

use defmt::{debug, info};
use embassy_executor::task;

use embassy_time::Duration;

use firmware_protocol::{ImuType, SbPacket, SensorDataType, SensorStatus};
use futures_util::StreamExt;

pub type Quat = nalgebra::UnitQuaternion<f32>;

pub trait Imu {
	type Error: core::fmt::Debug;

	const IMU_TYPE: ImuType;
	fn quat(&mut self) -> nb::Result<Quat, Self::Error>;
}

#[task]
pub async fn imu_task(
	packets: &'static Packets,
	i2c: crate::aliases::ඞ::I2cConcrete<'static>,
	mut delay: crate::aliases::ඞ::DelayConcrete,
) -> ! {
	debug!("imu_task!");
	let main_imu = ඞ::new_imu(i2c, &mut delay);
	info!("Initialized IMU!");

	imu_loop(0, main_imu, packets).await
}

/// Run a single IMU indefinitely
async fn imu_loop<T: Imu>(sensor_id: u8, mut imu: T, packets: &'static Packets) -> ! {
	// We update hosts view of sensors when this changes
	let mut state = (false, *packets.connection.lock().await);
	// The throttle will ensure our IMU data is only sent at 100hz, instead of as fast as possible
	let mut throttle = embassy_time::Ticker::every(Duration::from_millis(10));

	loop {
		// Wait for next IMU update
		throttle.next().await;

		// Query IMU data
		let quat = nb2a(|| imu.quat()).await;

		// Update sensor status when the IMU returned quat changes or connection is updated
		let new_state = (quat.is_ok(), *packets.connection.lock().await);
		if state != new_state {
			state = new_state;

			// Move to firmware_protocol?
			let sensor_status = if quat.is_ok() {
				SensorStatus::Ok
			} else {
				SensorStatus::Offline
			};

			// Let the server know
			packets
				.serverbound
				.send(SbPacket::SensorInfo {
					sensor_id,
					sensor_status,
					sensor_type: T::IMU_TYPE,
				})
				.await;
		}

		// Got valid IMU pose, send it to server
		if let Ok(unit) = quat {
			let quat = unit.into_inner().into();

			packets
				.serverbound
				.send(SbPacket::RotationData {
					sensor_id,
					data_type: SensorDataType::Normal,
					quat,
					calibration_info: 0xFF,
				})
				.await;
		}
	}
}
