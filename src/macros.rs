#[macro_export]
macro_rules! paniq {
    ($($arg:tt)+) => {{
        log::error!($($arg)+);
        panic!($($arg)+);
    }}
}
