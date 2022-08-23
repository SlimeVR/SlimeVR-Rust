#![no_std]
#![no_main]
// Needed for embassy
#![feature(type_alias_impl_trait)]

extern crate alloc;

// Set up global heap allocator
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// Set up backtraces
use esp_backtrace as _;

use core::fmt::Write;
use embassy_executor::{task, Executor};
use embedded_hal_async::delay::DelayUs;
use esp32c3_hal::{
    clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, RtcCntl,
    UsbSerialJtag,
};
use riscv_rt::entry;
use static_cell::StaticCell;

#[entry]
fn main() -> ! {
    // Initialize the global allocator BEFORE you use it
    {
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }
    }

    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = RtcCntl::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.set_super_wdt_enable(false);
    rtc.set_wdt_enable(false);
    wdt0.disable();
    wdt1.disable();

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR
        .init(Executor::new())
        .run(move |spawner| spawner.spawn(async_main()).unwrap());
}

#[task]
async fn async_main() {
    let mut usb = esp32c3_hal::UsbSerialJtag;
    let mut i = 0;
    loop {
        writeln!(&mut usb, "hello world, i is {i}").unwrap();
        i += 1;
    }
}

#[task]
async fn counter() {
    let mut i = 0;
    let mut delay = embassy_time::Delay;
    let mut usb = UsbSerialJtag;
    loop {
        DelayUs::delay_ms(&mut delay, 1000).await.unwrap();
        writeln!(&mut usb, "hello world, i is {i}").unwrap();
        i += 1;
    }
}
