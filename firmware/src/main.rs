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

// use rtt_target::{rtt_init_print, rprintln};
use defmt::error;

#[entry]
fn main() -> ! {
    // Initialize the global allocator BEFORE you use it
    {
        const HEAP_SIZE: usize = 10240;
        static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }
    }

    // defmt_serial::defmt_serial(crate::logging::BufferedSerial::take().unwrap());
    // rtt_init_print!();

    let peripherals = Peripherals::take().unwrap();

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR.init(Executor::new()).run(move |spawner| {
        spawner.spawn(async_main(peripherals)).unwrap();
        spawner.spawn(sensor_data()).unwrap();
    });
}

#[task]
async fn async_main(p: Peripherals) {
    // defmt_serial::defmt_serial(crate::logging::BufferedSerial::take().unwrap());

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
        // rtc.set_super_wdt_enable(false);
        // rtc.set_wdt_enable(false);
        wdt0.disable();
        wdt1.disable();
    }

    // let mut usb = esp32c3_hal::UsbSerialJtag;
    let mut i = 0;
    loop {
        error!("In main(), i was {}", i);
        // rprintln!("In main(), i was {}", i);
        // writeln!(&mut usb, "in main(), i is {i}").unwrap();
        i += 1;
        yield_now().await
    }
}

#[task]
async fn sensor_data() {
    let mut i = 0;
    // let mut delay = embassy_time::Delay;
    // let mut usb = esp32c3_hal::UsbSerialJtag;
    loop {
        error!("In data(), i was {}", i);
        // rprintln!("In data(), i was {}", i);
        // writeln!(&mut usb, "in data(), i is {i}").unwrap();

        // DelayUs::delay_ms(&mut delay, 1000).await.unwrap();
        i += 1;
        yield_now().await
    }
}
