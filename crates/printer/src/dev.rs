/// The purpose of this macro is to make it very easy to search for stray output left over from
/// printf-style debugging.
#[macro_export]
macro_rules! printf_debug {
    ($($token:tt)*) => {
        eprintln!($($token)*)
    }
}
