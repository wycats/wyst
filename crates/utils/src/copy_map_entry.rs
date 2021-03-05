pub use self::entry::*;
pub use self::entry_mut::*;

mod entry_mut {
    use crate::WystCopyMap;
    use wyst_core::{new, WystCopy};

    #[derive(Debug)]
    pub enum CopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        Occupied(OccupiedCopyMapEntryMut<'map, K, V>),
        Vacant(VacantCopyMapEntryMut<'map, K, V>),
    }

    impl<'map, K, V> CopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn occupied(map: &'map mut WystCopyMap<K, V>, key: K) -> CopyMapEntryMut<'map, K, V> {
            CopyMapEntryMut::Occupied(OccupiedCopyMapEntryMut { map, key })
        }

        pub fn vacant(map: &'map mut WystCopyMap<K, V>, key: K) -> CopyMapEntryMut<'map, K, V> {
            CopyMapEntryMut::Vacant(VacantCopyMapEntryMut { map, key })
        }

        pub fn extract<U>(self, update: impl FnOnce(V) -> (V, U)) -> Option<U> {
            match self {
                CopyMapEntryMut::Occupied(occupied) => Some(occupied.extract(update)),
                CopyMapEntryMut::Vacant(_) => {
                    // Do nothing. Use upsert instead if you want to be able to initialize and
                    // update.
                    None
                }
            }
        }

        pub fn upsert(self, create: impl FnOnce() -> V, update: impl FnOnce(V) -> V) {
            match self {
                CopyMapEntryMut::Occupied(occupied) => {
                    occupied.extract(|v| (update(v), ()));
                }
                CopyMapEntryMut::Vacant(vacant) => {
                    vacant.insert(update(create()));
                }
            }
        }

        pub fn insert(self, value: V) {
            match self {
                CopyMapEntryMut::Occupied(o) => o.insert(value),
                CopyMapEntryMut::Vacant(v) => v.insert(value),
            }
        }

        pub fn delete(self) -> Option<V> {
            match self {
                CopyMapEntryMut::Occupied(o) => o.delete(),
                CopyMapEntryMut::Vacant(_) => None,
            }
        }

        pub fn get(self) -> Option<V> {
            match self {
                CopyMapEntryMut::Occupied(o) => Some(o.get()),
                CopyMapEntryMut::Vacant(o) => o.get(),
            }
        }
    }

    #[derive(Debug, new)]
    pub struct OccupiedCopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        map: &'map mut WystCopyMap<K, V>,
        key: K,
    }

    impl<'map, K, V> OccupiedCopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn insert(self, value: V) {
            self.map.insert_entry(self.key, value)
        }

        pub fn extract<U>(self, update: impl FnOnce(V) -> (V, U)) -> U {
            let Self { key, map } = self;
            let value = *map
                .get_entry(self.key)
                .expect("OccupiedMapEntry must contain a value");
            let (update, extracted) = update(value);
            map.insert_entry(key, update);
            extracted
        }

        pub fn delete(self) -> Option<V> {
            self.map.delete_entry(self.key)
        }

        pub fn get(self) -> V {
            *self
                .map
                .get_entry(self.key)
                .expect("OccupiedMapEntry must contain a value")
        }
    }

    #[derive(Debug, new)]
    pub struct VacantCopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        map: &'map mut WystCopyMap<K, V>,
        key: K,
    }

    impl<'map, K, V> VacantCopyMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn insert(self, value: V) {
            self.map.insert_entry(self.key, value)
        }

        pub fn get(self) -> Option<V> {
            None
        }
    }
}

mod entry {
    use crate::WystCopyMap;
    use wyst_core::{new, WystCopy};

    #[derive(Debug)]
    pub enum CopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        Occupied(OccupiedCopyMapEntry<'map, K, V>),
        Vacant(VacantCopyMapEntry<'map, K, V>),
    }

    impl<'map, K, V> CopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn occupied(map: &'map WystCopyMap<K, V>, key: K) -> CopyMapEntry<'map, K, V> {
            CopyMapEntry::Occupied(OccupiedCopyMapEntry { map, key })
        }

        pub fn vacant(map: &'map WystCopyMap<K, V>, key: K) -> CopyMapEntry<'map, K, V> {
            CopyMapEntry::Vacant(VacantCopyMapEntry { map, key })
        }

        pub fn get(self) -> Option<V> {
            match self {
                CopyMapEntry::Occupied(o) => Some(*o.get()),
                CopyMapEntry::Vacant(o) => o.get(),
            }
        }
    }

    #[derive(Debug, new)]
    pub struct OccupiedCopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        map: &'map WystCopyMap<K, V>,
        key: K,
    }

    impl<'map, K, V> OccupiedCopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn get(self) -> &'map V {
            self.map
                .get_entry(self.key)
                .expect("OccupiedMapEntry must contain a value")
        }
    }

    #[derive(Debug, new)]
    pub struct VacantCopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        map: &'map WystCopyMap<K, V>,
        key: K,
    }

    impl<'map, K, V> VacantCopyMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystCopy,
    {
        pub fn get(self) -> Option<V> {
            None
        }
    }
}
