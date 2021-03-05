use std::ops::{self, RangeBounds};

use wyst_core::wyst_copy;

#[wyst_copy]
pub struct ResolvedRange<T>
where
    T: Copy,
{
    pub start: T,
    pub end: T,
}

impl<T> Into<ops::Range<T>> for ResolvedRange<T>
where
    T: Copy,
{
    fn into(self) -> ops::Range<T> {
        ops::Range {
            start: self.start,
            end: self.end,
        }
    }
}

#[wyst_copy]
pub enum RangeBound<T>
where
    T: Copy,
{
    Included(T),
    Excluded(T),
    Unbounded,
}

impl<T> RangeBound<T>
where
    T: Copy + ops::Add<usize, Output = T>,
{
    /// resolve_start resolves the bound as a starting bound. Since starting bounds are inclusive,
    /// [RangeBound::Included] is already correct, while [RangeBound::Excluded] needs to be
    /// incremented when resolved.
    pub fn resolve_start(self, default: T) -> T {
        match self {
            RangeBound::Included(bound) => bound,
            RangeBound::Excluded(bound) => bound + 1,
            RangeBound::Unbounded => default,
        }
    }

    /// resolve_start resolves the bound as an ending bound. Since ending bounds are exclusive,
    /// [RangeBound::Excluded] is already correct, while [RangeBound::Included] needs to be
    /// incremented when resolved.
    pub fn resolve_end(self, default: T) -> T {
        match self {
            RangeBound::Included(bound) => bound + 1,
            RangeBound::Excluded(bound) => bound,
            RangeBound::Unbounded => default,
        }
    }
}

macro_rules! adapt_std_bound {
    () => {
        adapt_std_bound!({}, {});
        adapt_std_bound!({ & }, { * });
    };

    ({ $($modifier:tt)* }, { $($deref:tt)* }) => {
        impl<T> From<ops::Bound<$($modifier)* T>> for RangeBound<T>
        where
            T: Copy,
        {
            fn from(bound: ops::Bound<$($modifier)* T>) -> Self {
                match bound {
                    ops::Bound::Included(bound) => RangeBound::Included($($deref)* bound),
                    ops::Bound::Excluded(bound) => RangeBound::Excluded($($deref)* bound),
                    ops::Bound::Unbounded => RangeBound::Unbounded,
                }
            }
        }
    };
}

adapt_std_bound!();

#[wyst_copy]
pub struct WystRange<T>
where
    T: Copy,
{
    pub start: RangeBound<T>,
    pub end: RangeBound<T>,
}

impl<T> WystRange<T>
where
    T: Copy + num::traits::Zero + ops::Add<usize, Output = T>,
{
    pub fn resolve(self, len: T) -> ResolvedRange<T> {
        let start = self.start.resolve_start(T::zero());
        let end = self.end.resolve_end(len);

        ResolvedRange { start, end }
    }
}

macro_rules! adapt_std_range {
    ($ns:tt :: $id:tt < $ty:tt >) => {
        impl From<$ns::$id<$ty>> for WystRange<$ty> {
            fn from(range: $ns::$id<$ty>) -> Self {
                WystRange {
                    start: range.start_bound().into(),
                    end: range.end_bound().into(),
                }
            }
        }
    };
}

adapt_std_range!(ops::RangeFrom<usize>);
adapt_std_range!(ops::RangeInclusive<usize>);
adapt_std_range!(ops::RangeTo<usize>);
adapt_std_range!(ops::RangeToInclusive<usize>);
adapt_std_range!(ops::Range<usize>);

impl From<ops::RangeFull> for WystRange<usize> {
    fn from(_: ops::RangeFull) -> Self {
        WystRange {
            start: RangeBound::Unbounded,
            end: RangeBound::Unbounded,
        }
    }
}
