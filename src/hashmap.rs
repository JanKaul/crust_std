use std::hash::{Hash, Hasher};
use std::mem;
use std::{collections::hash_map::DefaultHasher, slice::Iter};

use crate::{option::Option, owned_slice::OwnedSlice, owned_str::OwnedStr};

#[derive(Clone)]
#[repr(C)]
pub struct KeyValue<K, V> {
    key: K,
    value: V,
}

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
    hash(&s.as_bytes())
}

impl<K: Clone + Hash + Eq, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        let capacity = 64;
        let data = OwnedSlice::from(vec![Option::None; capacity]);
        HashMap {
            data,
            n_items: 0,
            capacity,
        }
    }

    pub fn iter(&self) -> Iter<Option<KeyValue<K, V>>> {
        self.data.iter()
    }

    pub fn insert(&mut self, key: &K, value: V) {
        if self.n_items > 3 * self.capacity / 4 {
            self.double_capacity()
        }
        let index = hash(key) % self.capacity as u64;
        let temp = self
            .data
            .iter_mut()
            .skip(index as usize)
            .find(|x| x.is_none());
        if let Some(entry) = temp {
            *entry = Option::Some(KeyValue {
                key: key.clone(),
                value,
            });
            self.n_items += 1;
        }
    }

    pub fn get(&mut self, key: &K) -> std::option::Option<&V> {
        let index = hash(key) % self.capacity as u64;
        self.data
            .iter()
            .cycle()
            .skip(index as usize)
            .find(|x| {
                if let Option::Some(entry) = x {
                    entry.key == *key
                } else {
                    false
                }
            })
            .and_then(|x| x.as_ref().map(|x| &x.value))
    }

    fn double_capacity(&mut self) {
        self.capacity *= 2;
        let new_data = OwnedSlice::from(vec![Option::None; self.capacity]);
        let old_data = mem::replace(&mut self.data, new_data);
        self.n_items = 0;
        for entry in old_data.into_iter().filter(|x| x.is_some()) {
            if let Option::Some(keyvalue) = entry {
                self.insert(&keyvalue.key, keyvalue.value)
            }
        }
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
        map.insert(&key1, 8);
        let val1 = map.get(&key1).expect("Failed to get value.");
        assert_eq!(*val1, 8);
        let key2: OwnedStr = "test2".to_string().into();
        map.insert(&key2, 16);
        let val2 = map.get(&key2).expect("Failed to get value.");
        assert_eq!(*val2, 16);
    }
}
