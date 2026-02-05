// Copyright (C) 2026 Stacks Open Internet Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

use clarity_types::types::QualifiedContractIdentifier;

use crate::vm::contracts::Contract;

const CONTRACT_AST_CACHE_SIZE: usize = 64 * 1024 * 1024;
const CONTRACT_AST_CACHE_ITEM_MAX_SIZE: usize = 1 * 1024 * 1024;

thread_local! {
    pub static CONTRACT_AST_CACHE: RefCell<ArenaLRUSizedCache<(QualifiedContractIdentifier, String), Contract>> = RefCell::new(ArenaLRUSizedCache::new(CONTRACT_AST_CACHE_SIZE, CONTRACT_AST_CACHE_ITEM_MAX_SIZE));
}

struct ArenaLRUSizedCacheNode<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct ArenaLRUSizedCache<K, V> {
    max_size: usize,
    max_item_size: usize,
    current_size: usize,
    arena: Vec<ArenaLRUSizedCacheNode<K, V>>,
    map: HashMap<K, usize>,
    sizes: Vec<usize>,
    head: Option<usize>, // MRU
    tail: Option<usize>, // LRU
    reusable_indices: Vec<usize>,
}

impl<K: Clone + Eq + Hash, V> ArenaLRUSizedCache<K, V> {
    pub fn new(max_size: usize, max_item_size: usize) -> Self {
        Self {
            max_size,
            max_item_size,
            current_size: 0,
            arena: vec![],
            map: HashMap::new(),
            sizes: vec![],
            head: None,
            tail: None,
            reusable_indices: vec![],
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn disconnect(&mut self, index: usize) {
        let prev = self.arena[index].prev;
        let next = self.arena[index].next;

        if let Some(prev_index) = prev {
            self.arena[prev_index].next = next;
        } else if self.head == Some(index) {
            self.head = next;
        }

        if let Some(next_index) = next {
            self.arena[next_index].prev = prev;
        } else if self.tail == Some(index) {
            self.tail = prev;
        }
    }

    fn set_mru(&mut self, index: usize) {
        if self.head != Some(index) {
            self.disconnect(index);

            self.arena[index].prev = None;
            self.arena[index].next = self.head;

            if let Some(previous_head_index) = self.head {
                self.arena[previous_head_index].prev = Some(index);
            }
            self.head = Some(index);

            if self.tail.is_none() {
                self.tail = Some(index);
            }
        }
    }

    pub fn get(&mut self, key: &K) -> Option<(&V, usize)> {
        if let Some(&index) = self.map.get(key) {
            self.set_mru(index);
            return Some((&self.arena[index].value, self.sizes[index]));
        }
        None
    }

    fn remove_by_index(&mut self, index: usize) {
        self.disconnect(index);
        self.map.remove(&self.arena[index].key);
        self.current_size -= self.sizes[index];
        self.reusable_indices.push(index);
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(&index) = self.map.get(&key) {
            self.remove_by_index(index);
        }
    }

    pub fn remove_lru(&mut self) {
        if let Some(index) = self.tail {
            self.remove_by_index(index);
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get_current_size(&self) -> usize {
        self.current_size
    }

    pub fn get_max_size(&self) -> usize {
        self.max_size
    }

    pub fn insert(&mut self, key: &K, value: V, size: usize) {
        // key already exists, set it as mru (we do not want to allow overwrites for performance reasons)
        if let Some(&index) = self.map.get(&key) {
            self.set_mru(index);
        } else {
            // check item size
            if size > self.max_item_size {
                return;
            }

            // check cache size
            if self.current_size + size > self.max_size {
                // remove a single item and retry, let's make life harder for cache poisoners...
                self.remove_lru();
                // not enough space, let's stop here
                if self.current_size + size > self.max_size {
                    return;
                }
            }

            // insert the new item
            let new_index = if let Some(index) = self.reusable_indices.pop() {
                self.arena[index] = ArenaLRUSizedCacheNode {
                    key: key.clone(),
                    value,
                    prev: None,
                    next: None,
                };
                self.sizes[index] = size;
                index
            } else {
                let index = self.arena.len();
                self.arena.push(ArenaLRUSizedCacheNode {
                    key: key.clone(),
                    value,
                    prev: None,
                    next: None,
                });
                self.sizes.push(size);
                index
            };

            self.map.insert(key.clone(), new_index);
            self.current_size += size;
            self.set_mru(new_index);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_initial_state() {
        let cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        assert_eq!(cache.max_size, 100);
        assert_eq!(cache.get_max_size(), 100);
        assert_eq!(cache.max_item_size, 10);
        assert_eq!(cache.current_size, 0);
        assert_eq!(cache.get_current_size(), 0);
        assert_eq!(cache.arena.len(), 0);
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.sizes.len(), 0);
        assert!(cache.head.is_none());
        assert!(cache.tail.is_none());
        assert_eq!(cache.reusable_indices.len(), 0);
    }

    #[test]
    fn test_non_existing_key() {
        let cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        assert!(!cache.contains_key(&String::from("test")));
    }

    #[test]
    fn test_existing_key() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key: String = String::from("test_key");
        cache.insert(&key, "test_value".into(), 10);
        assert!(cache.contains_key(&key));
    }

    #[test]
    fn test_item_size() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key: String = String::from("test_key");
        cache.insert(&key, "test_value".into(), 10);
        assert_eq!(cache.get(&key).unwrap().1, 10);
    }

    #[test]
    fn test_bad_item_size() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key: String = String::from("test_key");
        cache.insert(&key, "test_value".into(), 11);
        assert!(!cache.contains_key(&key));
        cache.insert(&key, "test_value".into(), 10);
        assert!(cache.contains_key(&key));
    }

    #[test]
    fn test_current_size() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        cache.insert(&key0, "test_value".into(), 10);
        assert_eq!(cache.current_size, 10);
        cache.insert(&key1, "test_value".into(), 10);
        assert_eq!(cache.current_size, 20);
        cache.insert(&key2, "test_value".into(), 10);
        assert_eq!(cache.current_size, 30);
    }

    #[test]
    fn test_lru() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        let key3: String = String::from("test_key3");

        cache.insert(&key0, "test_value".into(), 10);
        assert_eq!(cache.head, Some(0));
        assert_eq!(cache.tail, Some(0));
        cache.insert(&key1, "test_value".into(), 10);
        assert_eq!(cache.head, Some(1));
        assert_eq!(cache.tail, Some(0));
        cache.insert(&key2, "test_value".into(), 10);
        assert_eq!(cache.head, Some(2));
        assert_eq!(cache.tail, Some(0));

        let _ = cache.get(&key0);
        assert_eq!(cache.head, Some(0));
        assert_eq!(cache.tail, Some(1));

        let _ = cache.get(&key2);
        assert_eq!(cache.head, Some(2));
        assert_eq!(cache.tail, Some(1));

        let _ = cache.get(&key1);
        assert_eq!(cache.head, Some(1));
        assert_eq!(cache.tail, Some(0));

        cache.insert(&key3, "test_value".into(), 10);
        assert_eq!(cache.head, Some(3));
        assert_eq!(cache.tail, Some(0));
    }

    #[test]
    fn test_max_size_lru() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(20, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        cache.insert(&key0, "test_value".into(), 10);
        assert!(cache.contains_key(&key0));
        cache.insert(&key1, "test_value".into(), 10);
        assert!(cache.contains_key(&key1));
        cache.insert(&key2, "test_value".into(), 10);
        assert!(cache.contains_key(&key2));
        assert!(!cache.contains_key(&key0)); // expected to be the LRU
        assert!(cache.contains_key(&key1));

        cache.insert(&key0, "test_value".into(), 10);

        assert!(cache.contains_key(&key0));
        assert!(!cache.contains_key(&key1)); // expected to be the LRU
        assert!(cache.contains_key(&key2));
    }

    #[test]
    fn test_max_size() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(20, 10);
        let key0: String = String::from("test_key0");

        cache.insert(&key0, "test_value".into(), 21);
        assert!(!cache.contains_key(&key0));

        assert_eq!(cache.arena.len(), 0);
    }

    #[test]
    fn test_remove_lru() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        cache.insert(&key0, "test_value".into(), 10);
        cache.insert(&key1, "test_value".into(), 10);
        cache.insert(&key2, "test_value".into(), 10);

        assert_eq!(cache.len(), 3);

        cache.remove_lru();

        assert_eq!(cache.len(), 2);

        assert_eq!(cache.head, Some(2));
        assert_eq!(cache.tail, Some(1));
    }

    #[test]
    fn test_remove_tail() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        cache.insert(&key0, "test_value".into(), 10);
        cache.insert(&key1, "test_value".into(), 10);
        cache.insert(&key2, "test_value".into(), 10);

        assert_eq!(cache.len(), 3);

        cache.remove(&key0);

        assert_eq!(cache.len(), 2);

        assert_eq!(cache.head, Some(2));
        assert_eq!(cache.tail, Some(1));
    }

    #[test]
    fn test_remove_head() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        cache.insert(&key0, "test_value".into(), 10);
        cache.insert(&key1, "test_value".into(), 10);
        cache.insert(&key2, "test_value".into(), 10);

        assert_eq!(cache.len(), 3);

        cache.remove(&key2);

        assert_eq!(cache.len(), 2);

        assert_eq!(cache.head, Some(1));
        assert_eq!(cache.tail, Some(0));
    }

    #[test]
    fn test_remove_non_existent() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");
        let key1: String = String::from("test_key1");
        let key2: String = String::from("test_key2");
        let key3: String = String::from("test_key3");

        cache.insert(&key0, "test_value".into(), 10);
        cache.insert(&key1, "test_value".into(), 10);
        cache.insert(&key2, "test_value".into(), 10);

        assert_eq!(cache.len(), 3);

        cache.remove(&key3);

        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_remove_from_empty() {
        let mut cache = ArenaLRUSizedCache::<String, String>::new(100, 10);
        let key0: String = String::from("test_key0");

        assert_eq!(cache.len(), 0);

        cache.remove(&key0);

        assert_eq!(cache.len(), 0);
    }
}
