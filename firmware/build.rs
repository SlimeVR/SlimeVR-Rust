use feature_utils::mandatory_and_unique;

mandatory_and_unique!("mcu-esp32c3");
mandatory_and_unique!("imu-mpu6050");
mandatory_and_unique!("log-rtt", "log-usb-serial");

fn main() {}
