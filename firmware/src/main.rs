#![no_std]
#![no_main]
extern crate alloc;

// Set up global heap allocator
#[global_allocator]
static GLOBAL: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// Set up backtraces
use esp_backtrace as _;

use core::fmt::Write;
use esp32c3_hal::{
    clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc,
};
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let mut usb = esp32c3_hal::UsbSerialJtag;
    let mut i = 0;
    let arc = alloc::sync::Arc::new(10);
    loop {
        writeln!(&mut usb, "ayyyyy {i} {arc}");
        i += 1;
    }
}
