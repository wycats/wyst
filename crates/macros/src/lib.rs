pub use pretty_assertions;
pub use wyst_proc_macros::{unit_test, wyst_data};

#[macro_export]
macro_rules! unit_tests {
    (all({ $($all:tt)* }), $tests:ident($(( $name:tt, { $($tokens:tt)* } )),*)) => {
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

    ($tests:ident($($tokens:tt)*)) => {
        unit_tests!(all({}), $tests($($tokens)*));
    }
}
