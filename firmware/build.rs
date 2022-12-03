use feature_utils::mandatory_and_unique;

mandatory_and_unique!("mcu-esp32c3", "mcu-nrf52840");
mandatory_and_unique!("imu-mpu6050");
mandatory_and_unique!("log-rtt", "log-usb-serial");
mandatory_and_unique!("net-wifi", "net-stubbed");

fn main() {
	#[cfg(feature = "net-wifi")]
	println!("cargo:rustc-link-arg=-Tesp32c3_rom_functions.x"); // esp-wifi
}
