use crate::aliases::{Delay, I2c};
use crate::imu::{Imu, Quat};
use crate::utils;

use bno080::interface::I2cInterface;
use bno080::interface::SensorInterface;
use defmt::{debug, trace, warn};
use firmware_protocol::ImuType;

pub const IMU_REPORT_INTERVAL_MS: u16 = 10;
pub const I2C_ADDR: u8 = ::bno080::interface::i2c::ALTERNATE_ADDRESS;

pub type DriverError<I2c> = ::bno080::wrapper::WrapperError<
	<I2cInterface<I2c> as SensorInterface>::SensorError,
>;

struct Bno080<I: I2c> {
	driver: ::bno080::wrapper::BNO080<I2cInterface<I>>,
}
impl<I: crate::aliases::I2c> Bno080<I> {
	pub fn new(
		i2c: I,
		delay: &mut impl crate::aliases::Delay,
	) -> Result<Self, DriverError<I>> {
		let interface = ::bno080::interface::I2cInterface::new(i2c, I2C_ADDR);
		let mut driver = ::bno080::wrapper::BNO080::new_with_interface(interface);

		let mut last_err = Ok(());
		for i in 0..4 {
			delay.delay_ms(100u32);
			if let Err(e) = driver.soft_reset() {
				warn!(
					"Error resetting bno080 (atmpt {}): {:?}",
					i + 1,
					defmt::Debug2Format(&e)
				);
				last_err = Err(e);
			} else {
				last_err = Ok(());
				break;
			}
		}
		last_err?;

		let mut last_err = Ok(());
		for i in 0..4 {
			delay.delay_ms(500u32);
			if let Err(e) = driver.init(delay) {
				warn!(
					"Error initializing bno080 (atmpt {}): {:?}",
					i + 1,
					defmt::Debug2Format(&e)
				);
				last_err = Err(e);
			} else {
				last_err = Ok(());
				break;
			}
		}
		last_err?;

		let mut last_err = Ok(());
		for i in 0..4 {
			if let Err(e) = driver.enable_rotation_vector(IMU_REPORT_INTERVAL_MS) {
				warn!(
					"Error enabling rotation (atmpt {}): {:?}",
					i + 1,
					defmt::Debug2Format(&e)
				);
				last_err = Err(e);
			} else {
				last_err = Ok(());
				break;
			}
		}
		last_err?;

		// utils::retry(
		// 	4,
		// 	(),
		// 	|_| -> Result<(), ((), DriverError<I>)> {
		// 		// delay.delay_ms(400u32);
		// 		// trace!("Flushing comms");
		// 		// let _ = i2c.write(I2C_ADDR, &[0]);
		//
		// 		// delay.delay_ms(400u32);
		// 		trace!("Constructing IMU");
		//
		// 		// if let Err(e) = driver.init(delay) {
		// 		// 	return Err((driver.free().free(), e));
		// 		// }
		// 		debug!("Initialized bno080 driver");
		// 		// delay.delay_ms(100u32);
		// 		// if let Err(e) = driver.enable_rotation_vector(IMU_REPORT_INTERVAL_MS) {
		// 		// 	return Err((driver.free().free(), e));
		// 		// }
		// 		driver
		// 			.enable_rotation_vector(IMU_REPORT_INTERVAL_MS)
		// 			.map_err(|e| ((), e))?;
		// 		debug!("Enabled rotation vector");
		// 		Ok(())
		// 	},
		// 	|i| warn!("Retrying IMU connection (attempts so far: {})", i + 1),
		// )
		// .map_err(|(_, e)| panic!("{:?}", e))?;
		Ok(Self { driver })
	}
}

impl<I: I2c> Imu for Bno080<I> {
	type Error = DriverError<I>;

	const IMU_TYPE: ImuType = ImuType::Bno080;

	fn quat(&mut self) -> nb::Result<super::Quat, Self::Error> {
		let [i, j, k, w] = self.driver.rotation_quaternion()?;
		let q = nalgebra::Quaternion {
			coords: nalgebra::vector![i, j, k, w],
		};
		// TODO: This is already normalized, we can use unsafe for performance
		Ok(Quat::from_quaternion(q))
	}
}

pub fn new_imu(i2c: impl I2c, delay: &mut impl Delay) -> impl Imu {
	Bno080::new(i2c, delay).expect("Failed to initialize bno080")
}
