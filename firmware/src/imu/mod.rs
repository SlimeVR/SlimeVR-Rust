mod driver;
mod fusion;

use defmt::{debug, info, trace, warn};
use embassy_executor::task;
use embedded_hal::blocking::delay::DelayMs;

use crate::{
	aliases::ඞ::{DelayConcrete, I2cConcrete},
	utils::Unreliable,
};

pub use driver::*;
pub use fusion::*;

pub type Quat = nalgebra::UnitQuaternion<f32>;
pub type Vec3 = nalgebra::Vector3<f32>;

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
	// We'll do that later
	let mut imu = fusion::Naive::new(
		driver::Bmi160::new(i2c, &mut delay).expect("Failed to initialize IMU"),
	);
	info!("Initialized IMU!");

	loop {
		let q = match imu.quat().await {
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
	}
}
