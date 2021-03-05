mod empty;

use std::{fmt::Debug, hash::Hash};

pub use empty::WystEmpty;

pub trait WystData: Debug + Clone + Hash + Eq {}

impl<T> WystData for T where T: Debug + Clone + Hash + Eq {}

pub trait WystCopy: WystData + Copy {}

impl<T> WystCopy for T where T: WystData + Copy {}

pub trait WystDataValue: Debug + Eq {}

impl<T> WystDataValue for T where T: Debug + Eq {}
