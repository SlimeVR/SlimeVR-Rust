mod stubbed;

#[cfg(feature = "imu-stubbed")]
pub mod ඞ {
	pub use crate::imu::drivers::stubbed::*;
}

#[cfg(feature = "imu-mpu6050")]
#[path = "mpu6050.rs"]
pub mod ඞ;

#[cfg(feature = "imu-bmi160")]
#[path = "bmi160/mod.rs"]
pub mod ඞ;
