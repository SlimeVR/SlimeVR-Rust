#![no_std]

/// Asserts that no more than one of the comma-separated features is active at once.
// source: https://github.com/TheDan64/inkwell/blob/36c3b106e61b1b45295a35f94023d93d9328c76f/src/lib.rs#L81-L110
#[macro_export]
macro_rules! unique {
    () => {};
    ($first:literal $(,$rest:literal)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        $crate::unique!($($rest),*);
    }
}

/// Asserts that at least one of the comma-separated features is active.
// this one is mine :)
#[macro_export]
macro_rules! at_least_one_provided {
    ($($feature:literal),*) => {
        #[cfg(not(any(
            $(feature = $feature),*
        )))]
        compile_error!("You must provide one of the mandatory features!");
    }
}

/// Asserts that there is exactly one feature in the comma separated feature set active.
#[macro_export]
macro_rules! mandatory_and_unique {
    ($($feature:literal),*) => {
        $crate::at_least_one_provided!($($feature),*);
        $crate::unique!($($feature),*);
    }
}
