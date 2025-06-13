use cfg_aliases::cfg_aliases;
use eyre::{eyre, Result, WrapErr};
use feature_utils::mandatory_and_unique;
use serde::Deserialize;
use std::{
	env, fs,
	path::{self, Path, PathBuf},
};

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

fn main() -> Result<()> {
	// By default, Cargo will re-run a build script whenever
	// any rust file in the project changes. This ensures that these additional files
	// matter too.
	println!("cargo:rerun-if-changed=linker_scripts/");
	println!("cargo:rerun-if-changed=boards/");
	println!("cargo:rerun-if-changed=.env");
	println!("cargo:rerun-if-env-changed=BOARD");

	load_env_and_aliases();
	check_features_compatible();

	// Link into Espressif's radio driver blobs
	#[cfg(all(feature = "esp-wifi"))]
	println!("cargo:rustc-link-arg=-Trom_functions.x");

	memory_x!("mcu-nrf52832");
	memory_x!("mcu-nrf52840");

	let board_cfg = BoardConfig::from_file(&BoardConfig::get_path()?)?;
	board_cfg.apply_to_env();

	Ok(())
}

fn load_env_and_aliases() {
	let _ = dotenvy::dotenv();

	// NOTE: Can't use the `cfg_aliases` in the build script itself, only applies to
	// rest of codebase.
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
}

fn check_features_compatible() {
	mandatory_and_unique!("mcu-esp32", "mcu-esp32c3", "mcu-nrf52832", "mcu-nrf52840");
	mandatory_and_unique!("imu-stubbed", "imu-mpu6050", "imu-bmi160");
	mandatory_and_unique!("log-rtt", "log-usb-serial", "log-uart");
	mandatory_and_unique!("net-wifi", "net-ble", "net-stubbed");
	mandatory_and_unique!("fusion-stubbed", "fusion-dcm", "fusion-vqf");

	#[cfg(any(feature = "mcu-nrf52840", feature = "mcu-nrf52832"))]
	mandatory_and_unique!(
		"nrf-boot-none",
		"nrf-boot-mbr",
		"nrf-boot-s132",
		"nrf-boot-s140"
	);

	#[cfg(all(
		feature = "net-wifi",
		any(feature = "mcu-nrf52840", feature = "mcu-nrf52832")
	))]
	compile_error!("nrf52 mcu family doesn't support wifi!");

	#[cfg(all(feature = "mcu-nrf52832", feature = "log-usb-serial"))]
	compile_error!("the nrf52832 doesn't support USB!");
}

#[allow(dead_code)]
fn memoryx(memoryx: String) {
	#[allow(unused_variables)]
	let layout = MemoryLayout::RAW;
	#[cfg(feature = "nrf-boot-mbr")]
	let layout = MemoryLayout::MBR_ONLY;
	#[cfg(feature = "nrf-boot-s140")]
	let layout = MemoryLayout::S140;
	#[cfg(feature = "nrf-boot-s132")]
	let layout = MemoryLayout::S132;

	let memoryx = memoryx.replace(
		"APP_CODE_BASE",
		&format!("{:#x}", layout.sd_flash_size + layout.mbr_size),
	);
	let memoryx = memoryx.replace("SD_RAM_SIZE", &format!("{:#x}", layout.sd_ram_size));

	// panic!("{}", memoryx); // for debugging
	let out = path::PathBuf::from(env::var("OUT_DIR").unwrap());
	fs::write(out.join("memory.x"), memoryx).unwrap();
	println!("cargo:rustc-link-search={}", out.display());
}

/// Describes data to fill `memory.x` with
struct MemoryLayout {
	mbr_size: usize,
	sd_flash_size: usize,
	sd_ram_size: usize,
}
#[allow(dead_code)]
impl MemoryLayout {
	/// No MBR no bootloader no softdevice. Our code is the entry point.
	const RAW: MemoryLayout = MemoryLayout {
		mbr_size: 0x0,
		sd_flash_size: 0x0,
		sd_ram_size: 0x0,
	};
	/// Uses the Master Boot Record and bootloader, but not softdevice.
	const MBR_ONLY: MemoryLayout = MemoryLayout {
		mbr_size: 0x1000,
		sd_flash_size: 0x0,
		// TODO: Is this correct? Disabling softdevice requires 8 bytes, but idk what
		// should happen if softdevice is entirely overwritten with our firmware.
		sd_ram_size: 0x8,
	};
	/// Softdevice 140.
	const S140: MemoryLayout = MemoryLayout {
		mbr_size: 0x1000,
		sd_flash_size: 0x26000,
		sd_ram_size: 0x8,
	};
	/// Softdevice 132.
	const S132: MemoryLayout = MemoryLayout {
		mbr_size: 0x1000,
		sd_flash_size: 0x25000,
		sd_ram_size: 0x8,
	};
}

#[derive(Debug, Deserialize)]
struct BoardConfig {
	pins: Pins,
}
#[derive(Debug, Deserialize)]
struct Pins {
	scl: String,
	sda: String,
	int0: String,
	int1: String,
	tx: String,
	rx: String,
}
impl BoardConfig {
	/// Loads a board config from a file
	fn from_file(p: &Path) -> Result<Self> {
		let s = std::fs::read_to_string(p)
			.wrap_err(format!("Failed to read board toml: {}", p.display()))?;
		toml::from_str(&s).wrap_err("Failed to deserialize board toml")
	}
	/// Gets the path to the board config, or errors if we can't pick one.
	fn get_path() -> Result<PathBuf> {
		std::env::var("BOARD").map(PathBuf::from).or_else(|_| {
			Self::default_path().ok_or(eyre!(
				"Please specify a board toml in the `BOARD` environment variable!"
			))
		})
	}
	/// What the default board config should be, if any
	fn default_path() -> Option<PathBuf> {
		let mut boards_dir =
			PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
		boards_dir.push("boards");
		#[allow(unused)]
		let result: Option<PathBuf> = None;

		#[cfg(feature = "mcu-esp32c3")]
		let result = Some(boards_dir.join("xiao_esp32c3.toml"));
		#[cfg(feature = "mcu-esp32")]
		let result = Some(boards_dir.join("devkitv1_esp32.toml"));
		#[cfg(feature = "mcu-nrf52840")]
		let result = Some(boards_dir.join("xiao_sense.toml"));
		#[cfg(feature = "mcu-nrf52832")]
		let result = Some(boards_dir.join("nrf52832_tmp.toml"));

		result
	}

	/// Applies the board config to cargo's environment variables
	fn apply_to_env(&self) {
		macro_rules! set_var {
			($var:literal, $field:ident) => {
				println!("cargo:rustc-env={}={}", $var, self.pins.$field);
			};
		}
		set_var!("PIN_SCL", scl);
		set_var!("PIN_SDA", sda);
		set_var!("PIN_INT0", int0);
		set_var!("PIN_INT1", int1);
		set_var!("PIN_TX", tx);
		set_var!("PIN_RX", rx);
	}
}
