extern crate alloc;

// Set up global heap allocator
#[cfg(feature = "mcu-esp32c3")]
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[cfg(feature = "mcu-nrf52840")]
#[global_allocator]
static ALLOCATOR: alloc_cortex_m::CortexMHeap = alloc_cortex_m::CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
	panic!("Allocation failed, out of memory");
}
// #[cfg(feature = "mcu-nrf52840")]
// #[alloc_error_handler]
// fn oom(_: core::alloc::Layout) -> ! {
// 	loop {}
// }

// Set up backtraces
// use esp_backtrace as _;

use panic_defmt as _;

// Set up global defmt logger
#[cfg(all(
	any(feature = "mcu-esp32c3"),
	any(feature = "log-usb-serial", feature = "log-uart")
))]
use defmt_esp_println as _;
#[cfg(feature = "log-rtt")]
use defmt_rtt as _;

/// Sets up any global state
pub fn setup() {
	// Initialize the global allocator BEFORE you use it
	{
		const HEAP_SIZE: usize = 10 * 1024;
		static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

		unsafe {
			#[cfg(feature = "mcu-esp32c3")]
			ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE);
			#[cfg(feature = "mcu-nrf52840")]
			ALLOCATOR.init(HEAP.as_mut_ptr() as usize, HEAP_SIZE);
		}
	}
}

/// This will be called when a hardware exception occurs
#[cfg(feature = "mcu-esp32c3")]
#[export_name = "ExceptionHandler"]
pub fn custom_exception_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
	use defmt::error;

	let mepc = riscv::register::mepc::read();
	let mcause = riscv::register::mcause::read();
	let mtval = riscv::register::mtval::read();
	#[cfg(feature = "mcu-esp32c3")]
	{
		let backtrace = esp_backtrace::arch::backtrace();
		for e in backtrace {
			if let Some(addr) = e {
				error!("0x{:x}", addr);
			}
		}
	}
	error!("Unexpected hardware exception.");
	panic!(
		"MCAUSE: {:?}, RA: {:#x}, MEPC: {:#b} MTVAL: {:#x}",
		mcause.cause(),
		trap_frame.ra,
		mepc,
		mtval
	);
}
