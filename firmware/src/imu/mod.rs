mod drivers;
mod fusion;

use defmt::{debug, info, trace, warn};
use embassy_executor::task;
use embassy_futures::yield_now;
use firmware_protocol::ImuType;

use crate::{
	aliases::ඞ::{DelayConcrete, I2cConcrete},
	utils::{nb2a, Unreliable},
};

pub type Quat = nalgebra::UnitQuaternion<f32>;

pub trait FusedImu {
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
	mut delay: impl crate::aliases::Delay,
) -> ! {
	debug!("Imu task");
	let mut imu = new_imu(i2c, &mut delay);
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

fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl crate::aliases::Delay,
) -> impl FusedImu {
	use crate::imu::drivers as d;

	#[cfg(feature = "imu-bmi160")]
	return d::bmi160::new_imu(i2c, delay);
	#[cfg(feature = "imu-mpu6050")]
	return d::mpu6050::new_imu(i2c, delay);
	#[cfg(feature = "imu-stubbed")]
	return d::stubbed::new_imu(i2c, delay);
}
