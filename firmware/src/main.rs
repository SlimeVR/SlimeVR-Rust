#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

mod aliases;
mod globals;
mod imu;
mod peripherals;
mod utils;

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
        spawner.spawn(imu_task(p.i2c, p.delay)).unwrap();
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
async fn imu_task(
    i2c: crate::aliases::I2cConcrete,
    delay: crate::aliases::DelayConcrete,
) {
    crate::imu::imu_task(i2c, delay).await
}
