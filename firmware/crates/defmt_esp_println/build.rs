// source: https://github.com/TheDan64/inkwell/blob/36c3b106e61b1b45295a35f94023d93d9328c76f/src/lib.rs#L81-L110
macro_rules! assert_unique_features {
    () => {};
    ($first:literal $(,$rest:literal)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_features!($($rest),*);
    }
}

// this one is mine :)
macro_rules! assert_one_provided {
    ($($feature:literal),*) => {
        #[cfg(not(any(
            $(feature = $feature),*
        )))]
        compile_error!("You must provide one of the mandatory features!");
    }
}

macro_rules! features {
    ($($feature:literal),*) => {
        assert_one_provided!($($feature),*);
        assert_unique_features!($($feature),*);
    }
}
features!("uart", "jtag_serial");
features!("esp32", "esp32c2", "esp32c3", "esp32s2", "esp32s3", "esp8266");

fn main() {}
