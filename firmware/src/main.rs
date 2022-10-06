#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

mod aliases;
mod globals;
mod imu;
mod networking;
mod peripherals;
mod utils;

use defmt::{debug, info};
use embassy_executor::{task, Executor};
use riscv_rt::entry;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    self::globals::setup();
    info!("Booted");

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
    crate::networking::wifi::network_task().await
}

#[task]
async fn imu_task(
    i2c: crate::aliases::I2cConcrete,
    delay: crate::aliases::DelayConcrete,
) {
    crate::imu::imu_task(i2c, delay).await
}
