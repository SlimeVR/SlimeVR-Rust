#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

mod aliases;
mod globals;
mod imu;
mod peripherals;

use crate::imu::Imu;
use crate::imu::Mpu6050;

use defmt::error;
use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use riscv_rt::entry;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    self::globals::setup();

    let p = self::peripherals::get_peripherals();

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR.init(Executor::new()).run(move |spawner| {
        spawner.spawn(async_main()).unwrap();
        spawner.spawn(sensor_data(p.i2c, p.delay)).unwrap();
    });
}

#[task]
async fn async_main() {
    let mut i = 0;
    loop {
        error!("In main(), i was {}", i);
        i += 1;
        yield_now().await // Yield to ensure fairness
    }
}

#[task]
async fn sensor_data(
    i2c: crate::aliases::I2cConcrete,
    mut delay: crate::aliases::DelayConcrete,
) {
    let mut imu = Mpu6050::new(i2c, &mut delay).expect("Failed to initialize MPU");
    let mut i = 0;
    loop {
        error!("In data(), i was {}", i);
        i += 1;
        let q = nb2a(|| imu.quat()).await.expect("Fatal IMU Error");
        error!(
            "Quat values: x: {}, y: {}, z: {}, w: {}",
            q.coords.x, q.coords.y, q.coords.z, q.coords.w
        );
        yield_now().await // Yield to ensure fairness
    }
}

/// Converts a nb::Result to an async function by looping and yielding to the async
/// executor.
async fn nb2a<T, E>(mut f: impl FnMut() -> nb::Result<T, E>) -> Result<T, E> {
    loop {
        let v = f();
        match v {
            Ok(t) => return Ok(t),
            Err(nb::Error::Other(e)) => return Err(e),
            Err(nb::Error::WouldBlock) => yield_now().await,
        }
    }
}
