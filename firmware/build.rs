use std::{env, fs, path};

use cfg_aliases::cfg_aliases;
use feature_utils::mandatory_and_unique;

mandatory_and_unique!("mcu-esp32", "mcu-esp32c3", "mcu-nrf52832", "mcu-nrf52840");
mandatory_and_unique!("imu-stubbed", "imu-mpu6050", "imu-bmi160");
mandatory_and_unique!("log-rtt", "log-usb-serial", "log-uart");
mandatory_and_unique!("net-wifi", "net-stubbed");

/// Use memory.x.feature file as memory map
macro_rules! memory_x {
	($mcu:literal) => {
		#[cfg(feature = $mcu)]
		{
			let memoryx_content =
				String::from(include_str!(concat!("linker_scripts/memory.x.", $mcu)));
			memoryx(memoryx_content)
		}
	};
}

fn main() {
	#[cfg(all(feature = "mcu-nrf52832", feature = "log-usb-serial"))]
	compile_error!("the nrf52832 doesn't support USB!");

	cfg_aliases! {
		mcu_f_nrf52: { any(feature = "mcu-nrf52840", feature = "mcu-nrf52832") },
		mcu_f_esp32: { any(feature = "mcu-esp32", feature = "mcu-esp32c3") },
		bbq: { all(
			any(mcu_f_nrf52),
			any(feature = "log-uart", feature = "log-usb-serial")
		)},
		cortex_m: { mcu_f_nrf52 },
		xtensa: { any(feature = "mcu-esp32") },
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

#[allow(dead_code)]
fn memoryx(memoryx: String) {
	#[allow(unused_variables)]
	let sd_info = SoftdeviceInfo::NONE;
	#[cfg(feature = "softdevice-140")]
	let sd_info = SoftdeviceInfo::S140;
	#[cfg(feature = "softdevice-132")]
	let sd_info = SoftdeviceInfo::S132;

	let memoryx = memoryx.replace(
		"APP_CODE_BASE",
		&format!("{:#x}", sd_info.sd_flash_size + sd_info.mbr_size),
	);
	let memoryx =
		memoryx.replace("SD_RAM_SIZE", &format!("{:#x}", sd_info.sd_ram_size));

	let out = path::PathBuf::from(env::var("OUT_DIR").unwrap());
	fs::write(out.join("memory.x"), memoryx).unwrap();
	println!("cargo:rustc-link-search={}", out.display());
}

struct SoftdeviceInfo {
	mbr_size: usize,
	sd_flash_size: usize,
	sd_ram_size: usize,
}
#[allow(dead_code)]
impl SoftdeviceInfo {
	const NONE: SoftdeviceInfo = SoftdeviceInfo {
		mbr_size: 0x1000,
		sd_flash_size: 0x0,
		sd_ram_size: 0x0,
	};
	const S140: SoftdeviceInfo = SoftdeviceInfo {
		mbr_size: 0x1000,
		sd_flash_size: 0x26000,
		sd_ram_size: 0x0,
	};
	const S132: SoftdeviceInfo = SoftdeviceInfo {
		mbr_size: 0x1000,
		sd_flash_size: 0x25000,
		sd_ram_size: 0x0,
	};
	// TODO: Support other softdevice versions
}
