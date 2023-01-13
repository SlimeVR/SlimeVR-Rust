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

use defmt::{debug, info, trace, warn};
use embassy_executor::task;
use embassy_futures::yield_now;
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;

use crate::{
	aliases::ඞ::{DelayConcrete, I2cConcrete},
	utils::{nb2a, Unreliable},
};

pub type Quat = nalgebra::UnitQuaternion<f32>;

pub trait Imu {
	type Error: core::fmt::Debug;

	const IMU_TYPE: ImuType;
	// TODO: This should be async
	fn quat(&mut self) -> nb::Result<Quat, Self::Error>;
}

/// Gets data from the IMU
#[task]
pub async fn imu_task(
	quat_signal: &'static Unreliable<Quat>,
	i2c: I2cConcrete<'static>,
	delay: DelayConcrete,
) -> ! {
	imu_task_inner(quat_signal, i2c, delay).await
}

/// Same as [`imu_task()`] but this version's arguments are type erased behind impl
/// Trait to avoid accidentally accessing concrete behavior.
async fn imu_task_inner(
	quat_signal: &Unreliable<Quat>,
	i2c: impl crate::aliases::I2c,
	mut delay: impl DelayMs<u32>,
) -> ! {
	debug!("Imu task");
	let mut imu = ඞ::new_imu(i2c, &mut delay);
	info!("Initialized IMU!");

	loop {
		let q = match nb2a(|| imu.quat()).await {
			Ok(q) => q,
			Err(err) => {
				warn!("Error in IMU: {}", defmt::Debug2Format(&err));
				continue;
			}
		};
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
