#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

mod aliases;
mod globals;
mod imu;
pub use self::globals::ehal;

use crate::imu::Mpu6050;
use crate::{aliases::I2cConcrete, imu::Imu};

use defmt::error;
use ehal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use fugit::RateExtU32;
use riscv_rt::entry;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    self::globals::setup();

    let p = Peripherals::take().unwrap();

    let mut system = p.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    {
        let mut rtc = Rtc::new(p.RTC_CNTL);
        let timer_group0 = TimerGroup::new(p.TIMG0, &clocks);
        let mut wdt0 = timer_group0.wdt;
        let timer_group1 = TimerGroup::new(p.TIMG1, &clocks);
        let mut wdt1 = timer_group1.wdt;

        rtc.rwdt.disable();
        rtc.swd.disable();
        wdt0.disable();
        wdt1.disable();
    }

    let io = ehal::IO::new(p.GPIO, p.IO_MUX);
    // let hz =
    let i2c = ehal::i2c::I2C::new(
        p.I2C0,
        io.pins.gpio10,
        io.pins.gpio8,
        400u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .expect("Failed to set up i2c");

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR.init(Executor::new()).run(move |spawner| {
        spawner.spawn(async_main()).unwrap();
        spawner
            .spawn(sensor_data(i2c, ehal::Delay::new(&clocks)))
            .unwrap();
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
async fn sensor_data(i2c: I2cConcrete, mut delay: ehal::Delay) {
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
