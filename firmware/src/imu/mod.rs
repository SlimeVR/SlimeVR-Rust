mod drivers;
mod fusion;

use approx::AbsDiffEq;
use defmt::{debug, info, trace, warn};
use embassy_executor::task;
use firmware_protocol::ImuType;

use crate::{
	aliases::ඞ::{DelayConcrete, I2cConcrete},
	utils::Unreliable,
};

pub type Quat = nalgebra::UnitQuaternion<f32>;
pub type Accel = nalgebra::Vector3<f32>;
pub type Gyro = nalgebra::Vector3<f32>;

pub struct UnfusedData {
	pub accel: Accel,
	pub gyro: Gyro,
}

pub struct FusedData {
	pub q: Quat,
}

pub trait Imu {
	type Error: core::fmt::Debug; // TODO: Maybe use defmt instead?
	/// The data that the imu outputs.
	type Data;

	const IMU_TYPE: ImuType;
	/// Performs IO to get the next data from the imu.
	async fn next_data(&mut self) -> Result<Self::Data, Self::Error>;
}

/// Gets data from the IMU
#[task]
pub async fn imu_task(
	quat_signal: &'static Unreliable<Quat>,
	i2c: I2cConcrete<'static>,
	mut delay: DelayConcrete,
) -> ! {
	debug!("Imu task");

	let mut imu = new_imu(i2c, &mut delay);
	info!("Initialized IMU!");

	let mut i = 0;
	let mut prev_q = Quat::identity();
	loop {
		let q = match imu.next_data().await {
			Ok(q) => q.q,
			Err(err) => {
				warn!("Error in IMU: {}", defmt::Debug2Format(&err));
				continue;
			}
		};
		if i % 1000 == 0 {
			trace!(
				"Quat values: x: {}, y: {}, z: {}, w: {}",
				q.coords.x,
				q.coords.y,
				q.coords.z,
				q.coords.w
			);
		}
		i += 1;
		if !q.abs_diff_eq(&prev_q, 0.0001) {
			prev_q = q;
			quat_signal.signal(q);
		}
	}
}

fn new_imu(
	i2c: impl crate::aliases::I2c,
	delay: &mut impl crate::aliases::Delay,
) -> impl Imu<Data = FusedData> {
	use crate::imu::drivers as d;

	#[cfg(feature = "imu-bmi160")]
	return d::bmi160::new_imu(i2c, delay);
	#[cfg(feature = "imu-mpu6050")]
	return d::mpu6050::new_imu(i2c, delay);
	#[cfg(feature = "imu-stubbed")]
	return d::stubbed::new_imu(i2c, delay);
}
