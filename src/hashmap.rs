use serde::de::{MapAccess, Visitor};
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::hash_map::DefaultHasher, slice::Iter};
use std::{fmt, mem};

use crate::{option::Option, owned_slice::OwnedSlice, owned_str::OwnedStr};

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct KeyValue<K, V> {
    key: K,
    value: V,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct HashMap<K, V> {
    data: OwnedSlice<Option<KeyValue<K, V>>>,
    n_items: usize,
    capacity: usize,
}

#[inline]
fn hash<K: Hash>(s: &K) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[no_mangle]
pub extern "C" fn crust_hash_owned_str(s: &OwnedStr) -> u64 {
    hash(s)
}

impl<K: Clone + Hash + Eq, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        let capacity = 64;
        Self::with_capacity(capacity)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let data = OwnedSlice::from(vec![Option::None; capacity]);
        HashMap {
            data,
            n_items: 0,
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.n_items
    }

    pub fn insert(&mut self, key: K, value: V) -> std::option::Option<V> {
        if self.n_items > 3 * self.capacity / 4 {
            self.double_capacity()
        }
        let index = hash(&key) % self.capacity as u64;
        let temp = if let Some(temp) = self
            .data
            .iter_mut()
            .skip(index as usize)
            .find(|x| x.is_none())
        {
            temp
        } else {
            self.data
                .iter_mut()
                .find(|x| x.is_none())
                .expect("Failed to find empty slot in hashmap.")
        };
        let result = if temp.is_some() {
            mem::replace(temp, Option::None)
        } else {
            Option::None
        };
        *temp = Option::Some(KeyValue {
            key: key.clone(),
            value,
        });
        self.n_items += 1;
        result.map(|x| x.value).into()
    }

    pub fn get<Q>(&mut self, key: &Q) -> std::option::Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = hash(&key) % self.capacity as u64;
        self.data
            .iter()
            .cycle()
            .skip(index as usize)
            .find(|x| {
                if let Option::Some(entry) = x {
                    *entry.key.borrow() == *key
                } else {
                    false
                }
            })
            .and_then(|x| x.as_ref().map(|x| &x.value))
    }

    pub fn iter(&self) -> Iter<Option<KeyValue<K, V>>> {
        self.data.iter()
    }

    fn double_capacity(&mut self) {
        self.capacity *= 2;
        let new_data = OwnedSlice::from(vec![Option::None; self.capacity]);
        let old_data = mem::replace(&mut self.data, new_data);
        self.n_items = 0;
        for entry in old_data
            .into_iter()
            .filter_map(|x| Into::<std::option::Option<KeyValue<K, V>>>::into(x))
        {
            self.insert(entry.key, entry.value)
                .expect("Capacity should be enough.");
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone> Serialize for HashMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for entry in self
            .data
            .iter()
            .filter_map(|x| Into::<std::option::Option<&KeyValue<K, V>>>::into(x.as_ref()))
        {
            map.serialize_entry(&entry.key, &entry.value)?;
        }
        map.end()
    }
}

struct MapVisitor<K, V> {
    marker: PhantomData<fn() -> HashMap<K, V>>,
}

impl<K, V> MapVisitor<K, V> {
    fn new() -> Self {
        MapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
where
    K: Deserialize<'de> + Clone + Hash + Eq,
    V: Deserialize<'de> + Clone,
{
    type Value = HashMap<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(1));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<'de, K, V> Deserialize<'de> for HashMap<K, V>
where
    K: Deserialize<'de> + Clone + Hash + Eq,
    V: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MapVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::owned_str::OwnedStr;

    use super::HashMap;

    #[test]
    fn hashmap_insert_get() {
        let mut map = HashMap::new();
        let key1: OwnedStr = "test1".to_string().into();
        map.insert(key1.clone(), 8);
        let val1 = map.get(&key1).expect("Failed to get value.");
        assert_eq!(*val1, 8);
        let key2: OwnedStr = "test2".to_string().into();
        map.insert(key2.clone(), 16);
        let val2 = map.get(&key2).expect("Failed to get value.");
        assert_eq!(*val2, 16);
    }
}
