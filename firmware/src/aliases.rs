#[cfg(feature = "esp32c3")]
mod ඞ {
    pub use esp32c3_hal::ehal;
    pub type I2cConcrete = esp32c3_hal::i2c::I2C<esp32c3_hal::pac::I2C0>;
    pub use esp32c3_hal::Delay as DelayConcrete;
}

pub use ඞ::{ehal, DelayConcrete, I2cConcrete};

pub trait I2c:
    ehal::blocking::i2c::Write<Error = <Self as I2c>::Error>
    + ehal::blocking::i2c::WriteRead<Error = <Self as I2c>::Error>
{
    type Error: core::fmt::Debug;
}
impl<
        T: ehal::blocking::i2c::Write<Error = E>
            + ehal::blocking::i2c::WriteRead<Error = E>,
        E: core::fmt::Debug,
    > I2c for T
{
    type Error = E;
}
