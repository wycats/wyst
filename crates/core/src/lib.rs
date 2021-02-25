pub use pretty_assertions;
pub use wyst_core_traits::WystData;
pub use wyst_proc_macros::{new, unit_test, wyst_copy, wyst_data, wyst_display, Display};

#[macro_export]
macro_rules! unit_tests {
    (all({ $($all:tt)* }), $tests:ident($(( $name:tt, { $($tokens:tt)* } )),* $(,)*) $(,)*) => {
        #[cfg(test)]
        mod $tests {
            #[allow(unused)]
            use super::*;

            use $crate::unit_test;
            use $crate::pretty_assertions::assert_eq;

            $($all)*

            $(
                $crate::unit_test! { $name, || { $($tokens)* } }
            )*
        }
    };

    ($tests:ident($($tokens:tt)*) $(,)*) => {
        unit_tests!(all({}), $tests($($tokens)*));
    }
}
