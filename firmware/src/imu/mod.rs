mod mpu6050;
pub use self::mpu6050::Mpu6050;

pub type Quat = nalgebra::UnitQuaternion<f32>;

#[derive(Debug, Eq, PartialEq)]
pub enum ImuKind {
    Mpu6050,
}

pub trait Imu {
    type Error;

    const IMU_KIND: ImuKind;
    fn quat(&mut self) -> nb::Result<Quat, Self::Error>;
}
