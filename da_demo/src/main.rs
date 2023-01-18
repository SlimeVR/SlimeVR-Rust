#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_halt as _;

use defmt::info;

use cortex_m_rt::entry;


#[entry]
fn main() -> ! {
    loop {
        info!("hello");
    }
}
