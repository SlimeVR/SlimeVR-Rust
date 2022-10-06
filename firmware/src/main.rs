#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

mod aliases;
mod globals;
mod imu;
mod peripherals;
mod utils;

use crate::imu::Imu;
use crate::imu::Mpu6050;
use crate::utils::nb2a;

use defmt::{debug, trace};
use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use riscv_rt::entry;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    self::globals::setup();
    debug!("Booted");

    let p = self::peripherals::get_peripherals();
    debug!("Initialized peripherals");
    p.delay.delay(1000);

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR.init(Executor::new()).run(move |spawner| {
        spawner.spawn(network_task()).unwrap();
        spawner.spawn(sensor_task(p.i2c, p.delay)).unwrap();
    });
}

#[task]
async fn network_task() {
    debug!("Started network_task");
    let mut i = 0;
    loop {
        trace!("In main(), i was {}", i);
        i += 1;
        yield_now().await // Yield to ensure fairness
    }
}

#[task]
async fn sensor_task(
    i2c: crate::aliases::I2cConcrete,
    mut delay: crate::aliases::DelayConcrete,
) {
    debug!("Started sensor_task");
    let mut imu = Mpu6050::new(i2c, &mut delay).expect("Failed to initialize MPU6050");
    debug!("Initialized IMU!");

    let mut i = 0;
    loop {
        trace!("In data(), i was {}", i);
        i += 1;
        let q = nb2a(|| imu.quat()).await.expect("Fatal IMU Error");
        trace!(
            "Quat values: x: {}, y: {}, z: {}, w: {}",
            q.coords.x,
            q.coords.y,
            q.coords.z,
            q.coords.w
        );
        yield_now().await // Yield to ensure fairness
    }
}
