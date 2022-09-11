extern crate alloc;

// Set up global heap allocator
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// Set up backtraces
use esp_backtrace as _;

// Set up global defmt logger
use defmt_rtt as _;

// Choose the embedded hal based on the hardware (for now its just esp32c3)
pub use esp32c3_hal as ehal;

/// Sets up any global state
pub fn setup() {
    // Initialize the global allocator BEFORE you use it
    {
        const HEAP_SIZE: usize = 10 * 1024;
        static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }
    }
}
