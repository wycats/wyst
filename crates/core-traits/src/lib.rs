use std::{fmt::Debug, hash::Hash};

pub trait WystData: Debug + Clone + Hash + Eq {}

impl<T> WystData for T where T: Debug + Clone + Hash + Eq {}

pub trait WystCopy: WystData + Copy {}

impl<T> WystCopy for T where T: WystData + Copy {}
