// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;
use tracing::warn;

// Cache-aligned to avoid false sharing, similar to DashMap's design
#[repr(align(64))]
struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::Deref for CachePadded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Type aliases to reduce complexity
type Shard<K, V> = CachePadded<RwLock<BTreeMap<K, V>>>;
type ShardCollection<K, V> = Box<[Shard<K, V>]>;

/// A thread-safe wrapper around BTreeMap that uses sharding for better concurrent performance
/// Similar to DashMap's design but maintains ordering within each shard
pub struct ShardedConcurrentBTreeMap<K, V> {
    shards: ShardCollection<K, V>,
    shard_count: usize,
}

impl<K, V> Default for ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        let mut new_shards = Vec::with_capacity(self.shard_count);

        for shard in self.shards.iter() {
            let shard_map = match shard.read() {
                Ok(guard) => (*guard).clone(),
                Err(_) => {
                    warn!("Failed to read shard during clone, creating empty shard");
                    BTreeMap::new()
                }
            };
            new_shards.push(CachePadded::new(RwLock::new(shard_map)));
        }

        Self {
            shards: new_shards.into_boxed_slice(),
            shard_count: self.shard_count,
        }
    }
}

impl<K, V> ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash,
{
    /// Creates a new ShardedConcurrentBTreeMap with default shard count (16)
    pub fn new() -> Self {
        Self::with_shard_count(16)
    }

    /// Creates a new ShardedConcurrentBTreeMap with specified shard count
    pub fn with_shard_count(shard_count: usize) -> Self {
        let shard_count = shard_count.max(1); // Ensure at least 1 shard
        let mut shards = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            shards.push(CachePadded::new(RwLock::new(BTreeMap::new())));
        }

        Self {
            shards: shards.into_boxed_slice(),
            shard_count,
        }
    }

    /// Calculate which shard a key belongs to
    fn shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }

    /// Get the shard for a given key
    fn get_shard(&self, key: &K) -> &RwLock<BTreeMap<K, V>> {
        let index = self.shard_index(key);
        &self.shards[index].value
    }

    /// Inserts a key-value pair into the map
    /// Returns the previous value if the key was already present
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let shard = self.get_shard(&key);
        match shard.write() {
            Ok(mut guard) => guard.insert(key, value),
            Err(_) => {
                warn!(
                    "Failed to acquire write lock for shard in ShardedConcurrentBTreeMap::insert"
                );
                None
            }
        }
    }

    /// Gets a value by key, returning a cloned value if found
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let shard = self.get_shard(key);
        match shard.read() {
            Ok(guard) => guard.get(key).cloned(),
            Err(e) => {
                warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::get, error: {}", e);
                None
            }
        }
    }

    /// Removes a key-value pair from the map
    /// Returns the removed value if the key was present
    pub fn remove(&self, key: &K) -> Option<V> {
        let shard = self.get_shard(key);
        match shard.write() {
            Ok(mut guard) => guard.remove(key),
            Err(_) => {
                warn!(
                    "Failed to acquire write lock for shard in ShardedConcurrentBTreeMap::remove"
                );
                None
            }
        }
    }

    /// Checks if the map contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        let shard = self.get_shard(key);
        match shard.read() {
            Ok(guard) => guard.contains_key(key),
            Err(e) => {
                warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::contains_key, error: {}", e);
                false
            }
        }
    }

    /// Returns the number of elements in the map
    pub fn len(&self) -> usize {
        let mut total_len = 0;
        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => total_len += guard.len(),
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::len, error: {}", e);
                }
            }
        }
        total_len
    }

    /// Returns true if the map is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all elements from the map
    pub fn clear(&self) {
        for shard in self.shards.iter() {
            if let Ok(mut guard) = shard.write() {
                guard.clear();
            } else {
                warn!("Failed to acquire write lock for shard in ShardedConcurrentBTreeMap::clear");
            }
        }
    }

    /// Gets all keys in sorted order across all shards
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        let mut all_keys = Vec::new();

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    all_keys.extend(guard.keys().cloned());
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::keys, error: {}", e);
                }
            }
        }

        all_keys.sort();
        all_keys
    }

    /// Gets all values in key-sorted order across all shards
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn values(&self) -> Vec<V>
    where
        K: Clone,
        V: Clone,
    {
        let mut all_pairs = Vec::new();

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    all_pairs.extend(guard.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::values, error: {}", e);
                }
            }
        }

        all_pairs.sort_by(|a, b| a.0.cmp(&b.0));
        all_pairs.into_iter().map(|(_, v)| v).collect()
    }

    /// Gets all key-value pairs in key-sorted order across all shards
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn iter_cloned(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut all_pairs = Vec::new();

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    all_pairs.extend(guard.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::iter_cloned, error: {}", e);
                }
            }
        }

        all_pairs.sort_by(|a, b| a.0.cmp(&b.0));
        all_pairs
    }

    /// Retains only the elements specified by the predicate
    pub fn retain<F>(&self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        for shard in self.shards.iter() {
            if let Ok(mut guard) = shard.write() {
                guard.retain(|k, v| f(k, v));
            } else {
                warn!(
                    "Failed to acquire write lock for shard in ShardedConcurrentBTreeMap::retain"
                );
            }
        }
    }

    /// Executes a closure with read access to a specific shard
    pub fn with_shard_read<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&BTreeMap<K, V>) -> R,
    {
        let shard = self.get_shard(key);
        match shard.read() {
            Ok(guard) => Some(f(&*guard)),
            Err(e) => {
                warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::with_shard_read, error: {}", e);
                None
            }
        }
    }

    /// Executes a closure with write access to a specific shard
    pub fn with_shard_write<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&mut BTreeMap<K, V>) -> R,
    {
        let shard = self.get_shard(key);
        match shard.write() {
            Ok(mut guard) => Some(f(&mut *guard)),
            Err(_) => {
                warn!("Failed to acquire write lock for shard in ShardedConcurrentBTreeMap::with_shard_write");
                None
            }
        }
    }

    /// Gets the number of shards
    pub fn shard_count(&self) -> usize {
        self.shard_count
    }

    /// Gets range of key-value pairs within a specific shard
    /// This is more efficient than global range queries since it only locks one shard
    pub fn shard_range_cloned<T, R>(&self, key: &K, range: R) -> Vec<(K, V)>
    where
        T: Ord + ?Sized,
        K: std::borrow::Borrow<T> + Clone,
        V: Clone,
        R: std::ops::RangeBounds<T>,
    {
        let shard = self.get_shard(key);
        match shard.read() {
            Ok(guard) => guard
                .range(range)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            Err(e) => {
                warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::shard_range_cloned, error: {}", e);
                Vec::new()
            }
        }
    }
}

// Additional convenience methods for common use cases
impl<K, V> ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash + Clone,
    V: Clone,
{
    /// Extends the map with the contents of an iterator
    pub fn extend<I>(&self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

// Implement the standard FromIterator trait
impl<K, V> std::iter::FromIterator<(K, V)> for ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash + Clone,
    V: Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let map = Self::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_operations() {
        let map = ShardedConcurrentBTreeMap::new();

        // Test insert and get
        assert_eq!(map.insert("key1".to_string(), 1), None);
        assert_eq!(map.get(&"key1".to_string()), Some(1));

        // Test update
        assert_eq!(map.insert("key1".to_string(), 2), Some(1));
        assert_eq!(map.get(&"key1".to_string()), Some(2));

        // Test contains_key
        assert!(map.contains_key(&"key1".to_string()));
        assert!(!map.contains_key(&"key2".to_string()));

        // Test len and is_empty
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        // Test remove
        assert_eq!(map.remove(&"key1".to_string()), Some(2));
        assert_eq!(map.get(&"key1".to_string()), None);
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_sharding() {
        let map = ShardedConcurrentBTreeMap::with_shard_count(4);
        assert_eq!(map.shard_count(), 4);

        // Insert items that will go to different shards
        for i in 0..100 {
            map.insert(i, format!("value_{i}"));
        }

        assert_eq!(map.len(), 100);

        // Verify all items are accessible
        for i in 0..100 {
            assert_eq!(map.get(&i), Some(format!("value_{i}")));
        }
    }

    #[test]
    fn test_ordering_across_shards() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert in random order
        let values = vec![10, 5, 15, 3, 7, 12, 18, 1, 9, 14];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Keys should be in sorted order
        let keys = map.keys();
        let mut expected_keys = values.clone();
        expected_keys.sort();
        assert_eq!(keys, expected_keys);
    }

    #[test]
    fn test_concurrent_access() {
        let map = Arc::new(ShardedConcurrentBTreeMap::with_shard_count(8));
        let counter = Arc::new(AtomicUsize::new(0));
        let num_threads = 16;
        let num_operations = 100;

        let mut handles = vec![];

        // Spawn multiple threads doing concurrent operations
        for thread_id in 0..num_threads {
            let map_clone = Arc::clone(&map);
            let counter_clone = Arc::clone(&counter);

            let handle = thread::spawn(move || {
                for i in 0..num_operations {
                    let key = thread_id * num_operations + i;

                    // Insert
                    map_clone.insert(key, format!("value_{key}"));
                    counter_clone.fetch_add(1, Ordering::SeqCst);

                    // Read
                    let _value = map_clone.get(&key);

                    // Update
                    map_clone.insert(key, format!("updated_value_{key}"));
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all operations completed
        assert_eq!(counter.load(Ordering::SeqCst), num_threads * num_operations);
        assert_eq!(map.len(), num_threads * num_operations);

        // Verify ordering is maintained
        let keys = map.keys();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys);
    }

    #[test]
    fn test_shard_operations() {
        let map = ShardedConcurrentBTreeMap::new();

        for i in 1..=20 {
            map.insert(i, i * 10);
        }

        // Test shard-specific operations
        let key = 10;
        let shard_data = map.with_shard_read(&key, |btree| {
            btree.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
        });

        assert!(shard_data.is_some());
        let shard_data = shard_data.unwrap();

        // Verify the key we're looking for is in this shard
        assert!(shard_data.iter().any(|(k, _)| *k == key));
    }

    #[test]
    fn test_retain() {
        let map = ShardedConcurrentBTreeMap::new();

        for i in 1..=20 {
            map.insert(i, i * 10);
        }

        // Retain only even keys
        map.retain(|k, _v| *k % 2 == 0);

        assert_eq!(map.len(), 10);

        // Verify only even keys remain
        let keys = map.keys();
        for key in keys {
            assert_eq!(key % 2, 0);
        }
    }

    #[test]
    fn test_clone() {
        let map1 = ShardedConcurrentBTreeMap::new();

        for i in 1..=10 {
            map1.insert(i, format!("value_{i}"));
        }

        let map2 = map1.clone();

        // Both maps should have the same data
        assert_eq!(map1.len(), map2.len());

        for i in 1..=10 {
            assert_eq!(map1.get(&i), map2.get(&i));
        }

        // Modifications to one shouldn't affect the other
        map1.insert(11, "new_value".to_string());
        assert!(map1.contains_key(&11));
        assert!(!map2.contains_key(&11));
    }
}
