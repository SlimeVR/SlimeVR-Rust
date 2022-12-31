use cfg_aliases::cfg_aliases;
use feature_utils::mandatory_and_unique;

mandatory_and_unique!("mcu-esp32", "mcu-esp32c3", "mcu-nrf52832", "mcu-nrf52840");
mandatory_and_unique!("imu-stubbed", "imu-mpu6050", "imu-bmi160");
mandatory_and_unique!("log-rtt", "log-usb-serial", "log-uart");
mandatory_and_unique!("net-wifi", "net-stubbed");

/// Use memory.x.feature file as memory map
macro_rules! memory_x {
	($feature:literal) => {
		#[cfg(feature = $feature)]
		{
			use std::{env, fs, path};
			let out = path::PathBuf::from(env::var("OUT_DIR").unwrap());
			fs::write(
				out.join("memory.x"),
				include_bytes!(concat!("linker_scripts/memory.x.", $feature)),
			)
			.unwrap();
			println!("cargo:rustc-link-search={}", out.display());
		}
	};
}

fn main() {
	#[cfg(all(feature = "mcu-nrf52832", feature = "log-usb-serial"))]
	compile_error!("the nrf52832 doesn't support USB!");

	cfg_aliases! {
		mcu_f_nrf52: { any(feature = "mcu-nrf52840", feature = "mcu-nrf52832") },
		esp_xtensa: { any(feature = "mcu-esp32") },
		esp_riscv: { any(feature = "mcu-esp32c3") },
		esp: { any(esp_xtensa, esp_riscv) },
		bbq: { all(
			any(feature = "mcu-nrf52840", feature = "mcu-nrf52832"),
			any(feature = "log-uart", feature = "log-usb-serial")
		)},
		cortex_m: { any(feature = "mcu-nrf52840", feature = "mcu-nrf52832") },
		riscv: { any(feature = "mcu-esp32c3") },
	}

	#[cfg(all(feature = "net-wifi", feature = "mcu-esp32c3"))]
	println!("cargo:rustc-link-arg=-Tesp32c3_rom_functions.x"); // esp-wifi
	#[cfg(all(feature = "net-wifi", feature = "mcu-esp32"))]
	println!("cargo:rustc-link-arg=-Tesp32_rom_functions.x"); // esp-wifi

	// By default, Cargo will re-run a build script whenever
	// any file in the project changes. By specifying `memory.x`
	// here, we ensure the build script is only re-run when
	// `memory.x` is changed.
	println!("cargo:rerun-if-changed=linker_scripts/");

	memory_x!("mcu-nrf52832");
	memory_x!("mcu-nrf52840");
}
