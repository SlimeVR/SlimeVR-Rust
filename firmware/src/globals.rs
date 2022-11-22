extern crate alloc;

// Set up global heap allocator
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// Set up backtraces
// use esp_backtrace as _;

use defmt::debug;
use panic_defmt as _;

// Set up global defmt logger
#[cfg(all(
	any(feature = "mcu-esp32c3"),
	any(feature = "log-usb-serial", feature = "log-uart")
))]
use defmt_esp_println as _;
#[cfg(feature = "log-rtt")]
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

/// This will be called when a hardware exception occurs
#[export_name = "ExceptionHandler"]
pub fn custom_exception_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
	let mepc = riscv::register::mepc::read();
	let code = riscv::register::mcause::read().code() & 0xff;
	let mtval = riscv::register::mtval::read();
	let code = match code {
        0 => "Instruction address misaligned",
        1 => "Instruction access fault",
        2 => "Illegal instruction",
        3 => "Breakpoint",
        4 => "Load address misaligned",
        5 => "Load access fault",
        6 => "Store/AMO address misaligned",
        7 => "Store/AMO access fault",
        8 => "Environment call from U-mode",
        9 => "Environment call from S-mode",
        10 => "Reserved",
        11 => "Environment call from M-mode",
        12 => "Instruction page fault",
        13 => "Load page fault",
        14 => "Reserved",
        15 => "Store/AMO page fault",
        _ => "UNKNOWN",
    };
	
	#[cfg(feature = "mcu-esp32c3")]
	{
		let backtrace = esp_backtrace::arch::backtrace_internal(context.s0 as u32, 0);
		for e in backtrace {
			if let Some(addr) = e {
				debug!("0x{:x}", addr);
			}
		}
	}

	panic!(
		"Unexpected hardware exception.\nMCAUSE: {:?}, RA: {:#x}, MEPC: {:#b} MTVAL: {:#x}",
		code,
		trap_frame.ra,
		mepc,
		mtval,
	);

	loop {}
}
