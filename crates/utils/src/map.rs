use std::{fmt::Debug, hash::Hash};

use indexmap::IndexMap;
use wyst_core::prelude::*;

use crate::map_entry::{
    MapEntry, MapEntryMut, OccupiedMapEntry, OccupiedMapEntryMut, VacantMapEntry, VacantMapEntryMut,
};

#[derive(Eq, PartialEq, Clone)]
pub struct WystMap<K, V>
where
    K: WystCopy,
    V: WystData,
{
    pub(crate) inner: IndexMap<K, V>,
}

impl<K, V> WystEmpty for WystMap<K, V>
where
    K: WystCopy,
    V: WystData,
{
    fn empty() -> Self {
        WystMap {
            inner: IndexMap::new(),
        }
    }
}

impl<K, V> WystMap<K, V>
where
    K: WystCopy,
    V: WystData,
{
    pub fn entry_mut(&mut self, key: K) -> MapEntryMut<K, V> {
        match self.inner.get(&key) {
            Some(_) => MapEntryMut::Occupied(OccupiedMapEntryMut::new(self, key)),
            None => MapEntryMut::Vacant(VacantMapEntryMut::new(self, key)),
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, &'a V)> + 'a {
        self.inner.iter().map(|(k, v)| (*k, v))
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (K, &'a mut V)> + 'a {
        self.inner.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn entry<'inner>(&'inner self, key: K) -> MapEntry<'inner, K, V> {
        match self.inner.get(&key) {
            Some(_) => MapEntry::Occupied(OccupiedMapEntry::new(self, key)),
            None => MapEntry::Vacant(VacantMapEntry::new(self, key)),
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.entry(key).get()
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entry_mut(key).insert(value);
    }

    pub fn delete(&mut self, key: K) {
        self.entry_mut(key).delete();
    }

    pub(crate) fn insert_for_entry_api(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub(crate) fn delete_for_entry_api(&mut self, key: K) {
        self.inner.remove(&key);
    }

    pub(crate) fn get_for_entry_api(&self, key: K) -> Option<&V> {
        self.inner.get(&key)
    }

    pub(crate) fn get_mut_for_entry_api(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(&key)
    }
}

impl<K, V> Debug for WystMap<K, V>
where
    K: WystCopy,
    V: WystData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<K, V> Hash for WystMap<K, V>
where
    K: WystCopy,
    V: WystData,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in self.inner.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}
