use crate::ehal;

pub type I2cConcrete = ehal::i2c::I2C<ehal::pac::I2C0>;

pub trait I2c:
    ehal::ehal::blocking::i2c::Write<Error = <Self as I2c>::Error>
    + ehal::ehal::blocking::i2c::WriteRead<Error = <Self as I2c>::Error>
{
    type Error: core::fmt::Debug;
}
impl<
        T: ehal::ehal::blocking::i2c::Write<Error = E>
            + ehal::ehal::blocking::i2c::WriteRead<Error = E>,
        E: core::fmt::Debug,
    > I2c for T
{
    type Error = E;
}
