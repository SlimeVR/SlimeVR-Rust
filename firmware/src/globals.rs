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

// https://devzone.nordicsemi.com/f/nordic-q-a/52606/nrf52840-info-variant-interpretation

/// Sets up any global state
pub fn setup() {
	// https://github.com/probe-rs/probe-rs/issues/1324#issuecomment-1356273774
	// This is done because APProtect is enabled in some devices which basically
	// disables writing and reading the flash.
	// More info on the register:
	// https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.nrf52832.ps.v1.1/uicr.html
	#[cfg(feature = "mcu-nrf52840")] // TODO: Add nrf52832 support
	unsafe {
		use defmt::{error, info};
		use nrf52840_pac as pac;

		let ficr = &*pac::FICR::ptr();

		/// The hardware revision of the chip.
		/// See https://devzone.nordicsemi.com/f/nordic-q-a/55614/how-to-apply-nordic-software-workarounds-errata-for-a-given-hardware-revision-in-the-field
		#[derive(defmt::Format, Eq, PartialEq, Copy, Clone)]
		#[non_exhaustive]
		enum Revision {
			EngA,
			EngB,
			EngC,
			EngD,
			Rev1,
			Rev2,
			Rev3,
		}
		impl Revision {
			fn from_variant(bytes: [u8; 4]) -> Option<Revision> {
				let prefix = bytes[2];
				let suffix = bytes[3];

				let digit = suffix.is_ascii_digit();
				Some(match prefix {
					b'A' => Self::EngA,
					b'B' => Self::EngB,
					b'C' if !digit => Self::EngC,
					b'D' if !digit => Self::EngD,
					b'C' if digit => Self::Rev1,
					b'D' if digit => Self::Rev2,
					b'F' if digit => Self::Rev3,
					_ => return None,
				})
			}
		}
		// Get third character of the variant to determine hardware revision
		// https://infocenter.nordicsemi.com/index.jsp?topic=%2Fps_nrf52840%2Fficr.html&cp=4_0_0_3_3_0_8&anchor=register.INFO.VARIANT
		let code = ficr.info.variant.read().bits().to_be_bytes();
		let rev = Revision::from_variant(code);
		if let Some(rev) = rev {
			info!("Chip revision: {}", rev);
			if rev == Revision::Rev3 {
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
		} else {
			error!("Unknown hardware revision!");
		}
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
