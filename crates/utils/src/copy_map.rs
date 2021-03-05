use std::{fmt::Debug, hash::Hash};

use indexmap::IndexMap;
use wyst_core::{WystCopy, WystEmpty};

use crate::copy_map_entry::{CopyMapEntry, CopyMapEntryMut};

#[derive(Eq, PartialEq, Clone)]
pub struct WystCopyMap<K, V>
where
    K: WystCopy,
    V: WystCopy,
{
    pub(crate) inner: IndexMap<K, V>,
}

impl<K, V> WystEmpty for WystCopyMap<K, V>
where
    K: WystCopy,
    V: WystCopy,
{
    fn empty() -> Self {
        WystCopyMap {
            inner: IndexMap::new(),
        }
    }
}

impl<K, V> WystCopyMap<K, V>
where
    K: WystCopy,
    V: WystCopy,
{
    pub fn entry_mut(&mut self, key: K) -> CopyMapEntryMut<K, V> {
        match self.inner.get(&key) {
            Some(_) => CopyMapEntryMut::occupied(self, key),
            None => CopyMapEntryMut::vacant(self, key),
        }
    }

    pub fn entry<'inner>(&'inner self, key: K) -> CopyMapEntry<'inner, K, V> {
        match self.inner.get(&key) {
            Some(_) => CopyMapEntry::occupied(self, key),
            None => CopyMapEntry::vacant(self, key),
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, V)> + 'a {
        self.inner.iter().map(|(k, v)| (*k, *v))
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (K, &'a mut V)> + 'a {
        self.inner.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.entry(key).get()
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entry_mut(key).insert(value);
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        self.entry_mut(key).delete()
    }

    pub(crate) fn insert_entry(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub(crate) fn delete_entry(&mut self, key: K) -> Option<V> {
        self.inner.remove(&key)
    }

    pub(crate) fn get_entry(&self, key: K) -> Option<&V> {
        self.inner.get(&key)
    }
}

impl<K, V> Debug for WystCopyMap<K, V>
where
    K: WystCopy,
    V: WystCopy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<K, V> Hash for WystCopyMap<K, V>
where
    K: WystCopy,
    V: WystCopy,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in self.inner.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}
