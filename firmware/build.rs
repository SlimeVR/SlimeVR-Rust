use cfg_aliases::cfg_aliases;
use feature_utils::mandatory_and_unique;

mandatory_and_unique!("mcu-esp32", "mcu-esp32c3", "mcu-nrf52840");
mandatory_and_unique!("imu-stubbed", "imu-mpu6050");
mandatory_and_unique!("log-rtt", "log-usb-serial", "log-uart");
mandatory_and_unique!("net-wifi", "net-stubbed");

fn main() {
	cfg_aliases! {
		esp_xtensa: { any(feature = "mcu-esp32") },
		esp_riscv: { any(feature = "mcu-esp32c3") },
		esp: { any(esp_xtensa, esp_riscv) },
		bbq: { all(
			any(feature = "mcu-nrf52840"),
			any(feature = "log-uart", feature = "log-usb-serial")
		)},
		cortex_m: { any(feature = "mcu-nrf52840") },
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

	#[cfg(feature = "mcu-nrf52840")]
	{
		use std::{env, fs, path};
		let out = path::PathBuf::from(env::var("OUT_DIR").unwrap());
		fs::write(
			out.join("memory.x"),
			include_bytes!("linker_scripts/memory.x.nrf52840"),
		)
		.unwrap();
		// println!("cargo:rustc-link-arg=-Tmemory.x.nrf52840");
		println!("cargo:rustc-link-search={}", out.display());
	}
}
