use crate::imu::{FusedData, Imu, Quat};

use defmt::debug;
use embassy_time::{Duration, Ticker};
use embedded_hal::blocking::delay::DelayMs;
use firmware_protocol::ImuType;
use futures_util::StreamExt;

/// Fakes an IMU for easier testing.
struct FakeImu(Ticker);
impl FakeImu {
	pub fn new() -> Self {
		Self(Ticker::every(Duration::from_millis(5))) // 200Hz
	}
}

impl Imu for FakeImu {
	type Error = ();
	type Data = FusedData;

	const IMU_TYPE: ImuType = ImuType::Unknown(0xFF);

	async fn next_data(&mut self) -> Result<Self::Data, Self::Error> {
		self.0.next().await;
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
	FakeImu::new()
}
