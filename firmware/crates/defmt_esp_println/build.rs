use feature_utils::{mandatory_and_unique, unique};

unique!("uart", "jtag_serial");
mandatory_and_unique!("esp32", "esp32c2", "esp32c3", "esp32s2", "esp32s3", "esp8266");

fn main() {}
