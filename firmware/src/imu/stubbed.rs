use super::{Imu, ImuKind, Quat};

use defmt::debug;
use embedded_hal::blocking::delay::DelayMs;

/// Fakes an IMU for easier testing.
struct FakeImu;

impl Imu for FakeImu {
	type Error = ();

	const IMU_KIND: super::ImuKind = ImuKind::Mpu6050;

	fn quat(&mut self) -> nb::Result<Quat, Self::Error> {
		Ok(Quat::identity())
	}
}

#[allow(dead_code)]
pub fn new_imu(
	_i2c: impl crate::aliases::I2c,
	_delay: &mut impl DelayMs<u32>,
) -> impl crate::imu::Imu {
	debug!("Created FakeImu");
	FakeImu
}
