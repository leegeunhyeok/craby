/// Alias for `panic!` macro.
#[macro_export]
macro_rules! throw {
    ($($arg:tt)*) => {
        panic!($($arg)*)
    };
}

/// Catches a panic and returns a `Result` with the error message.
#[macro_export]
macro_rules! catch_panic {
    ($expr:expr) => {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $expr)).map_err(|e| {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };
            anyhow::anyhow!(msg)
        })
    };
}
