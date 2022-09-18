use super::{Imu, ImuKind, Quat};
use crate::aliases::{ehal, I2c};

use ehal::blocking::delay::DelayMs;
use mpu6050_dmp::address::Address;
use mpu6050_dmp::sensor::Mpu6050 as LibMpu;

pub struct Mpu6050<I: I2c> {
    mpu: LibMpu<I>,
    fifo_buf: [u8; 28],
}
impl<I: I2c> Mpu6050<I> {
    pub fn new(
        i2c: I,
        delay: &mut impl DelayMs<u32>,
    ) -> Result<Self, mpu6050_dmp::error::Error<I>> {
        let mut mpu = LibMpu::new(i2c, Address::default())?;
        mpu.initialize_dmp(delay)?;
        Ok(Self {
            mpu,
            fifo_buf: [0; 28],
        })
    }
}

impl<I: I2c> Imu for Mpu6050<I> {
    type Error = mpu6050_dmp::error::Error<I>;

    const IMU_KIND: super::ImuKind = ImuKind::Mpu6050;

    fn quat(&mut self) -> nb::Result<Quat, Self::Error> {
        if self.mpu.get_fifo_count()? >= 28 {
            let data = self.mpu.read_fifo(&mut self.fifo_buf)?;
            let data = &data[..16];
            let q = mpu6050_dmp::quaternion::Quaternion::from_bytes(data).unwrap();
            let q = nalgebra::Quaternion {
                coords: nalgebra::vector![q.x, q.y, q.z, q.w],
            };
            Ok(Quat::from_quaternion(q))
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
