mod stubbed;

#[cfg(feature = "imu-stubbed")]
mod ඞ {
	pub use crate::imu::stubbed::*;
}

#[cfg(feature = "imu-mpu6050")]
#[path = "mpu6050.rs"]
mod ඞ;

use crate::{networking::messaging::Signal, utils::nb2a};

use defmt::{debug, info, trace};
use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;

pub type Quat = nalgebra::UnitQuaternion<f32>;

#[derive(Debug, Eq, PartialEq)]
pub enum ImuKind {
	Mpu6050,
}

pub trait Imu {
	type Error: core::fmt::Debug;

	const IMU_KIND: ImuKind;
	fn quat(&mut self) -> nb::Result<Quat, Self::Error>;
}

/// Gets data from the IMU
pub async fn imu_task(
	quat_signal: &Signal<Quat>,
	i2c: impl crate::aliases::I2c,
	mut delay: impl DelayMs<u32>,
) {
	debug!("Started sensor_task");
	let mut imu = ඞ::new_imu(i2c, &mut delay);
	info!("Initialized IMU!");

	loop {
		let q = nb2a(|| imu.quat()).await.expect("Fatal IMU Error");
		trace!(
			"Quat values: x: {}, y: {}, z: {}, w: {}",
			q.coords.x,
			q.coords.y,
			q.coords.z,
			q.coords.w
		);
		quat_signal.signal(q);
		yield_now().await // Yield to ensure fairness
	}
}
