use crate::imu::{FusedData, Imu, Quat};

use defmt::debug;
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;

/// Fakes an IMU for easier testing.
struct FakeImu;

impl Imu for FakeImu {
	type Error = ();
	type Data = FusedData;

	const IMU_TYPE: ImuType = ImuType::Unknown(0xFF);

	async fn next_data(&mut self) -> Result<Self::Data, Self::Error> {
		Ok(FusedData {
			q: Quat::identity(),
		})
	}
}

#[allow(dead_code)]
pub fn new_imu(
	_i2c: impl crate::aliases::I2c,
	_delay: &mut impl DelayMs<u32>,
) -> impl Imu<Data = FusedData> {
	debug!("Created FakeImu");
	FakeImu
}
