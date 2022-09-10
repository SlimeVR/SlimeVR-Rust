#![no_std]
#![no_main]
// Needed for embassy macros
#![feature(type_alias_impl_trait)]

extern crate alloc;

// Set up global heap allocator
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// Set up backtraces
use esp_backtrace as _;

// Set up global defmt logger
use defmt_rtt as _;

use embassy_executor::{task, Executor};
use embassy_futures::yield_now;
use esp32c3_hal::{
    clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc,
};
use riscv_rt::entry;
use static_cell::StaticCell;

use defmt::error;

#[entry]
fn main() -> ! {
    // Initialize the global allocator BEFORE you use it
    {
        const HEAP_SIZE: usize = 10 * 1024;
        static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }
    }

    let peripherals = Peripherals::take().unwrap();

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR.init(Executor::new()).run(move |spawner| {
        spawner.spawn(async_main(peripherals)).unwrap();
        spawner.spawn(sensor_data()).unwrap();
    });
}

#[task]
async fn async_main(p: Peripherals) {
    let system = p.SYSTEM.split();
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

    let mut i = 0;
    loop {
        error!("In main(), i was {}", i);
        i += 1;
        yield_now().await
    }
}

#[task]
async fn sensor_data() {
    let mut i = 0;
    loop {
        error!("In data(), i was {}", i);
        i += 1;
        yield_now().await
    }
}
