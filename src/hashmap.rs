use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

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

#[no_mangle]
pub extern "C" fn crust_hash_owned_str(s: &OwnedStr) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(s.as_bytes());
    hasher.finish()
}

impl<V: Clone> HashMap<OwnedStr, V> {
    pub fn new() -> Self {
        let capacity = 64;
        let data = OwnedSlice::from(vec![
            Option::None as Option<KeyValue<OwnedStr, V>>;
            capacity
        ]);
        HashMap {
            data,
            n_items: 0,
            capacity: 64,
        }
    }

    pub fn insert(&mut self, key: &OwnedStr, value: V) {
        if self.n_items > 3 * self.capacity / 4 {}
        let index = crust_hash_owned_str(key) % self.capacity as u64;
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

    pub fn get(&mut self, key: &OwnedStr) -> std::option::Option<&V> {
        let index = crust_hash_owned_str(key) % self.capacity as u64;
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
}

#[cfg(test)]
mod tests {
    use super::HashMap;

    #[test]
    fn hashmap_insert_get() {
        let mut map = HashMap::new();
        let key1 = "test1".to_string().into();
        map.insert(&key1, 8);
        let val1 = map.get(&key1).expect("Failed to get value.");
        assert_eq!(*val1, 8);
        let key2 = "test2".to_string().into();
        map.insert(&key2, 16);
        let val2 = map.get(&key2).expect("Failed to get value.");
        assert_eq!(*val2, 16);
    }
}
