use wyst_core::prelude::*;

use crate::{map_entry::MapEntry, WystMap};

#[wyst_data]
pub struct WystSet<V>
where
    V: WystCopy,
{
    map: WystMap<V, bool>,
}

impl<V> WystEmpty for WystSet<V>
where
    V: WystCopy,
{
    fn empty() -> Self {
        WystSet {
            map: WystMap::empty(),
        }
    }
}

impl<V> WystSet<V>
where
    V: WystCopy,
{
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = V> + 'a {
        self.map.iter().map(|(k, _)| k)
    }

    pub fn add(&mut self, value: V) {
        self.map.insert(value, true)
    }

    pub fn delete(&mut self, value: V) {
        self.map.delete(value);
    }

    pub fn has(&self, value: V) -> bool {
        match self.map.entry(value) {
            MapEntry::Occupied(_) => true,
            MapEntry::Vacant(_) => false,
        }
    }
}
