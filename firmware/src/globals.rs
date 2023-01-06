extern crate alloc;

// Set up global heap allocator
#[cfg(mcu_f_esp32)]
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[cfg(cortex_m)]
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
#[cfg(all(mcu_f_esp32, any(feature = "log-usb-serial", feature = "log-uart")))]
use defmt_esp_println as _;
#[cfg(feature = "log-rtt")]
use defmt_rtt as _;

/// Sets up any global state
pub fn setup() {
	// https://github.com/probe-rs/probe-rs/issues/1324#issuecomment-1356273774
	// This is done because APProtect is enabled in some devices which basically
	// disables writing and reading the flash.
	// More info on the register:
	// https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.nrf52832.ps.v1.1/uicr.html
	#[cfg(mcu_f_nrf52)]
	unsafe {
		#[cfg(feature = "mcu-nrf52832")]
		use nrf52832_pac as pac;
		#[cfg(feature = "mcu-nrf52840")]
		use nrf52840_pac as pac;

		let nvmc = &*pac::NVMC::ptr();

		// UICR.APPROTECT = HwDisabled
		if *(0x10001208 as *mut u32) != 0x0000_005a {
			nvmc.config.write(|w| w.wen().wen());
			while nvmc.ready.read().ready().is_busy() {}
			core::ptr::write_volatile(0x10001208 as *mut u32, 0x0000_005a);
			while nvmc.ready.read().ready().is_busy() {}
			nvmc.config.reset();
			while nvmc.ready.read().ready().is_busy() {}
			cortex_m::peripheral::SCB::sys_reset();
		}

		// APPROTECT.DISABLE = SwDisabled
		(0x4000_0558 as *mut u32).write_volatile(0x0000_005a);
	}

	// Initialize the global allocator BEFORE you use it
	{
		const HEAP_SIZE: usize = 10 * 1024;
		static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

		unsafe {
			#[cfg(mcu_f_esp32)]
			ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE);
			#[cfg(cortex_m)]
			ALLOCATOR.init(HEAP.as_mut_ptr() as usize, HEAP_SIZE);
		}
	}
}

/// This will be called when a hardware exception occurs
#[cfg(target_arch = "riscv32")]
#[export_name = "ExceptionHandler"]
pub fn custom_exception_handler(trap_frame: &riscv_rt::TrapFrame) -> ! {
	use defmt::error;

	let mepc = riscv::register::mepc::read();
	let mcause = riscv::register::mcause::read();
	let mtval = riscv::register::mtval::read();
	#[cfg(riscv)]
	{
		let backtrace = esp_backtrace::arch::backtrace();
		for addr in backtrace.into_iter().flatten() {
			error!("0x{:x}", addr);
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

/// This will be called when a hardware exception occurs
#[cfg(xtensa)]
#[no_mangle]
#[link_section = ".rwtext"]
unsafe extern "C" fn __exception(
	cause: xtensa_lx_rt::exception::ExceptionCause,
	context: xtensa_lx_rt::exception::Context,
) {
	use defmt::error;

	let backtrace = esp_backtrace::arch::backtrace();
	for addr in backtrace.into_iter().flatten() {
		error!("0x{:x}", addr);
	}
	error!("Unexpected hardware exception.");
	panic!("Cause: {:?}, Ctx: {:?}", cause, context,);
}
