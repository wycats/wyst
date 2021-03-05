pub use self::entry::*;
pub use self::entry_mut::*;

mod entry_mut {
    use crate::WystMap;
    use wyst_core::{new, WystCopy, WystData};

    #[derive(Debug)]
    pub enum MapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        Occupied(OccupiedMapEntryMut<'map, K, V>),
        Vacant(VacantMapEntryMut<'map, K, V>),
    }

    impl<'map, K, V> MapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn occupied(map: &'map mut WystMap<K, V>, key: K) -> MapEntryMut<'map, K, V> {
            MapEntryMut::Occupied(OccupiedMapEntryMut { map, key })
        }

        pub fn vacant(map: &'map mut WystMap<K, V>, key: K) -> MapEntryMut<'map, K, V> {
            MapEntryMut::Vacant(VacantMapEntryMut { map, key })
        }

        pub fn extract<U>(self, extract: impl FnOnce(&mut V) -> U) -> Option<U> {
            match self {
                MapEntryMut::Occupied(mut occupied) => Some(occupied.extract(extract)),
                MapEntryMut::Vacant(_) => None,
            }
        }

        pub fn upsert(self, create: impl FnOnce() -> V, update: impl FnOnce(&mut V)) {
            match self {
                MapEntryMut::Occupied(mut occupied) => {
                    occupied.extract(update);
                }
                MapEntryMut::Vacant(vacant) => {
                    let mut value = create();
                    update(&mut value);
                    vacant.insert(value);
                }
            }
        }

        pub fn insert(self, value: V) {
            match self {
                MapEntryMut::Occupied(o) => o.insert(value),
                MapEntryMut::Vacant(v) => v.insert(value),
            }
        }

        pub fn delete(self) {
            match self {
                MapEntryMut::Occupied(o) => o.delete(),
                MapEntryMut::Vacant(_) => {}
            }
        }

        pub fn get(self) -> Option<&'map V> {
            match self {
                MapEntryMut::Occupied(o) => Some(o.get()),
                MapEntryMut::Vacant(o) => o.get(),
            }
        }
    }

    #[derive(Debug, new)]
    pub struct OccupiedMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        map: &'map mut WystMap<K, V>,
        key: K,
    }

    impl<'map, K, V> OccupiedMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn insert(self, value: V) {
            self.map.insert_for_entry_api(self.key, value)
        }

        /// Update the current value, extracting something from it. This is useful when the value is
        /// a collection, and you want to pull something out of the collection.
        pub fn extract<U>(&mut self, extract: impl FnOnce(&mut V) -> U) -> U {
            let value = self
                .map
                .get_mut_for_entry_api(self.key)
                .expect("OccupiedMapEntry must contain a value");
            extract(value)
        }

        pub fn delete(self) {
            self.map.delete_for_entry_api(self.key)
        }

        pub fn get(self) -> &'map mut V {
            self.map
                .get_mut_for_entry_api(self.key)
                .expect("OccupiedMapEntry must contain a value")
        }
    }

    #[derive(Debug, new)]
    pub struct VacantMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        map: &'map mut WystMap<K, V>,
        key: K,
    }

    impl<'map, K, V> VacantMapEntryMut<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn insert(self, value: V) {
            self.map.insert_for_entry_api(self.key, value)
        }

        pub fn get(self) -> Option<&'map V> {
            None
        }
    }
}

mod entry {
    use crate::WystMap;
    use wyst_core::{new, WystCopy, WystData};

    #[derive(Debug)]
    pub enum MapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        Occupied(OccupiedMapEntry<'map, K, V>),
        Vacant(VacantMapEntry<'map, K, V>),
    }

    impl<'map, K, V> MapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn occupied(map: &'map WystMap<K, V>, key: K) -> MapEntry<'map, K, V> {
            MapEntry::Occupied(OccupiedMapEntry { map, key })
        }

        pub fn vacant(map: &'map WystMap<K, V>, key: K) -> MapEntry<'map, K, V> {
            MapEntry::Vacant(VacantMapEntry { map, key })
        }

        pub fn get(self) -> Option<&'map V> {
            match self {
                MapEntry::Occupied(o) => Some(o.get()),
                MapEntry::Vacant(o) => o.get(),
            }
        }
    }

    #[derive(Debug, new)]
    pub struct OccupiedMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        map: &'map WystMap<K, V>,
        key: K,
    }

    impl<'map, K, V> OccupiedMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn get(self) -> &'map V {
            self.map
                .get_for_entry_api(self.key)
                .expect("OccupiedMapEntry must contain a value")
        }
    }

    #[derive(Debug, new)]
    pub struct VacantMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        map: &'map WystMap<K, V>,
        key: K,
    }

    impl<'map, K, V> VacantMapEntry<'map, K, V>
    where
        K: WystCopy,
        V: WystData,
    {
        pub fn get(self) -> Option<&'map V> {
            None
        }
    }
}
