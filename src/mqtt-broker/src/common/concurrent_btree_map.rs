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

/// Direction for iteration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterationDirection {
    Forward,
    Reverse,
}

/// Iterator for ShardedConcurrentBTreeMap that leverages BTreeMap's ordered properties
pub struct ShardedBTreeMapIterator<K, V> {
    // Use a simple approach: pre-sorted data but with optimized collection
    items: Vec<(K, V)>,
    current_index: usize,
}

impl<K, V> Iterator for ShardedBTreeMapIterator<K, V>
where
    K: Clone,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.items.len() {
            let (key, value) = &self.items[self.current_index];
            self.current_index += 1;
            Some((key.clone(), value.clone()))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len() - self.current_index;
        (remaining, Some(remaining))
    }
}

impl<K, V> ExactSizeIterator for ShardedBTreeMapIterator<K, V>
where
    K: Clone,
    V: Clone,
{
}

impl<K, V> ShardedBTreeMapIterator<K, V> {
    fn new(items: Vec<(K, V)>, direction: IterationDirection) -> Self {
        let mut sorted_items = items;
        if direction == IterationDirection::Reverse {
            sorted_items.reverse();
        }

        Self {
            items: sorted_items,
            current_index: 0,
        }
    }
}

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
    /// Optimized to leverage BTreeMap's ordered nature
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        // Pre-allocate vector with estimated capacity
        let mut all_keys = Vec::with_capacity(self.len());

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // BTreeMap keys() already provides ordered iteration
                    all_keys.extend(guard.keys().cloned());
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::keys, error: {}", e);
                }
            }
        }

        // Use unstable sort for better performance since we only care about final order
        all_keys.sort_unstable();
        all_keys
    }

    /// Gets all keys in forward order (smallest to largest) across all shards
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn keys_forward(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.keys() // Default keys() method already returns forward order
    }

    /// Gets all keys in reverse order (largest to smallest) across all shards
    /// Optimized to leverage BTreeMap's ordered nature
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn keys_reverse(&self) -> Vec<K>
    where
        K: Clone,
    {
        // Pre-allocate vector with estimated capacity
        let estimated_capacity = self.len();
        let mut all_keys = Vec::with_capacity(estimated_capacity);

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // Use rev() to iterate in reverse order within each shard
                    all_keys.extend(guard.keys().rev().cloned());
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::keys_reverse, error: {}", e);
                }
            }
        }

        // Sort in reverse order - use unstable sort for better performance
        all_keys.sort_unstable_by(|a, b| b.cmp(a));
        all_keys
    }

    /// Gets all keys with specified direction across all shards
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn keys_with_direction(&self, direction: IterationDirection) -> Vec<K>
    where
        K: Clone,
    {
        match direction {
            IterationDirection::Forward => self.keys_forward(),
            IterationDirection::Reverse => self.keys_reverse(),
        }
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
    /// Optimized to leverage BTreeMap's ordered nature and avoid unnecessary sorting
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn iter_cloned(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        // Pre-allocate vector with estimated capacity
        let estimated_capacity = self.len();
        let mut all_pairs = Vec::with_capacity(estimated_capacity);

        // Collect from all shards - each shard is already ordered
        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // BTreeMap iter() already provides ordered iteration
                    all_pairs.extend(guard.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::iter_cloned, error: {}", e);
                }
            }
        }

        // Sort only once at the end - this is more efficient than sorting within each shard
        all_pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
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

    /// Gets the minimum key across all shards
    /// Returns None if the map is empty
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn min_key(&self) -> Option<K>
    where
        K: Clone,
    {
        let mut min_key: Option<K> = None;

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // Use first_key_value which is more efficient than keys().next()
                    if let Some((shard_min, _)) = guard.first_key_value() {
                        match &min_key {
                            None => {
                                min_key = Some(shard_min.clone());
                            }
                            Some(current_min) => {
                                if shard_min < current_min {
                                    min_key = Some(shard_min.clone());
                                }
                            }
                        }
                    }
                    // If this shard is empty, continue to next shard without cloning
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::min_key, error: {}", e);
                }
            }
        }

        min_key
    }

    /// Gets the minimum key-value pair across all shards
    /// Returns None if the map is empty
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn min_key_value(&self) -> Option<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut min_pair: Option<(K, V)> = None;

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // Use first_key_value which is more efficient than iter().next()
                    if let Some((shard_min_key, shard_min_value)) = guard.first_key_value() {
                        match &min_pair {
                            None => {
                                min_pair = Some((shard_min_key.clone(), shard_min_value.clone()));
                            }
                            Some((current_min_key, _)) => {
                                if shard_min_key < current_min_key {
                                    min_pair =
                                        Some((shard_min_key.clone(), shard_min_value.clone()));
                                }
                            }
                        }
                    }
                    // If this shard is empty, continue to next shard without cloning
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::min_key_value, error: {}", e);
                }
            }
        }

        min_pair
    }

    /// Gets the maximum key across all shards
    /// Returns None if the map is empty
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn max_key(&self) -> Option<K>
    where
        K: Clone,
    {
        let mut max_key: Option<K> = None;

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // Use last_key_value which is more efficient than keys().next_back()
                    if let Some((shard_max, _)) = guard.last_key_value() {
                        match &max_key {
                            None => {
                                max_key = Some(shard_max.clone());
                            }
                            Some(current_max) => {
                                if shard_max > current_max {
                                    max_key = Some(shard_max.clone());
                                }
                            }
                        }
                    }
                    // If this shard is empty, continue to next shard without cloning
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::max_key, error: {}", e);
                }
            }
        }

        max_key
    }

    /// Gets the maximum key-value pair across all shards
    /// Returns None if the map is empty
    /// Note: This operation locks all shards sequentially, so it's expensive
    pub fn max_key_value(&self) -> Option<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let mut max_pair: Option<(K, V)> = None;

        for shard in self.shards.iter() {
            match shard.read() {
                Ok(guard) => {
                    // Use last_key_value which is more efficient than iter().next_back()
                    if let Some((shard_max_key, shard_max_value)) = guard.last_key_value() {
                        match &max_pair {
                            None => {
                                max_pair = Some((shard_max_key.clone(), shard_max_value.clone()))
                            }
                            Some((current_max_key, _)) => {
                                if shard_max_key > current_max_key {
                                    max_pair =
                                        Some((shard_max_key.clone(), shard_max_value.clone()));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to acquire read lock for shard in ShardedConcurrentBTreeMap::max_key_value, error: {}", e);
                }
            }
        }

        max_pair
    }

    /// Creates an iterator over the map in forward order (smallest to largest key)
    /// Note: This operation locks all shards sequentially and collects all data, so it's expensive
    pub fn iter_forward(&self) -> ShardedBTreeMapIterator<K, V>
    where
        K: Clone,
        V: Clone,
    {
        let items = self.iter_cloned();
        ShardedBTreeMapIterator::new(items, IterationDirection::Forward)
    }

    /// Creates an iterator over the map in reverse order (largest to smallest key)
    /// Note: This operation locks all shards sequentially and collects all data, so it's expensive
    pub fn iter_reverse(&self) -> ShardedBTreeMapIterator<K, V>
    where
        K: Clone,
        V: Clone,
    {
        let items = self.iter_cloned();
        ShardedBTreeMapIterator::new(items, IterationDirection::Reverse)
    }

    /// Creates an iterator with specified direction
    /// Note: This operation locks all shards sequentially and collects all data, so it's expensive
    pub fn iter_with_direction(
        &self,
        direction: IterationDirection,
    ) -> ShardedBTreeMapIterator<K, V>
    where
        K: Clone,
        V: Clone,
    {
        let items = self.iter_cloned();
        ShardedBTreeMapIterator::new(items, direction)
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

// Implement IntoIterator for direct use in for loops (forward iteration by default)
impl<K, V> IntoIterator for &ShardedConcurrentBTreeMap<K, V>
where
    K: Ord + Hash + Clone,
    V: Clone,
{
    type Item = (K, V);
    type IntoIter = ShardedBTreeMapIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_forward()
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

    #[test]
    fn test_min_key() {
        let map = ShardedConcurrentBTreeMap::new();

        // Test empty map
        assert_eq!(map.min_key(), None);

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Minimum key should be 10
        assert_eq!(map.min_key(), Some(10));

        // Remove the minimum key
        map.remove(&10);
        assert_eq!(map.min_key(), Some(20));

        // Remove more keys
        map.remove(&20);
        map.remove(&30);
        assert_eq!(map.min_key(), Some(40));

        // Clear the map
        map.clear();
        assert_eq!(map.min_key(), None);
    }

    #[test]
    fn test_min_key_value() {
        let map = ShardedConcurrentBTreeMap::new();

        // Test empty map
        assert_eq!(map.min_key_value(), None);

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, val * 100);
        }

        // Minimum key-value pair should be (10, 1000)
        assert_eq!(map.min_key_value(), Some((10, 1000)));

        // Remove the minimum key
        map.remove(&10);
        assert_eq!(map.min_key_value(), Some((20, 2000)));

        // Test with string keys
        let string_map = ShardedConcurrentBTreeMap::new();
        string_map.insert("zebra".to_string(), 1);
        string_map.insert("apple".to_string(), 2);
        string_map.insert("banana".to_string(), 3);

        assert_eq!(string_map.min_key_value(), Some(("apple".to_string(), 2)));
    }

    #[test]
    fn test_max_key() {
        let map = ShardedConcurrentBTreeMap::new();

        // Test empty map
        assert_eq!(map.max_key(), None);

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Maximum key should be 70
        assert_eq!(map.max_key(), Some(70));

        // Remove the maximum key
        map.remove(&70);
        assert_eq!(map.max_key(), Some(60));

        // Remove more keys
        map.remove(&60);
        map.remove(&50);
        assert_eq!(map.max_key(), Some(40));

        // Clear the map
        map.clear();
        assert_eq!(map.max_key(), None);
    }

    #[test]
    fn test_max_key_value() {
        let map = ShardedConcurrentBTreeMap::new();

        // Test empty map
        assert_eq!(map.max_key_value(), None);

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, val * 100);
        }

        // Maximum key-value pair should be (70, 7000)
        assert_eq!(map.max_key_value(), Some((70, 7000)));

        // Remove the maximum key
        map.remove(&70);
        assert_eq!(map.max_key_value(), Some((60, 6000)));

        // Test with string keys
        let string_map = ShardedConcurrentBTreeMap::new();
        string_map.insert("zebra".to_string(), 1);
        string_map.insert("apple".to_string(), 2);
        string_map.insert("banana".to_string(), 3);

        assert_eq!(string_map.max_key_value(), Some(("zebra".to_string(), 1)));
    }

    #[test]
    fn test_min_key_with_sharding() {
        // Test with multiple shards to ensure min_key works across shards
        let map = ShardedConcurrentBTreeMap::with_shard_count(8);

        // Insert many values that will be distributed across shards
        for i in (1..=100).rev() {
            map.insert(i, format!("value_{i}"));
        }

        // Minimum should still be 1
        assert_eq!(map.min_key(), Some(1));
        assert_eq!(map.min_key_value(), Some((1, "value_1".to_string())));

        // Remove some small values
        for i in 1..=10 {
            map.remove(&i);
        }

        // Now minimum should be 11
        assert_eq!(map.min_key(), Some(11));
        assert_eq!(map.min_key_value(), Some((11, "value_11".to_string())));
    }

    #[test]
    fn test_min_key_single_element() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert single element
        map.insert(42, "answer".to_string());

        assert_eq!(map.min_key(), Some(42));
        assert_eq!(map.min_key_value(), Some((42, "answer".to_string())));

        // Remove the only element
        map.remove(&42);
        assert_eq!(map.min_key(), None);
        assert_eq!(map.min_key_value(), None);
    }

    #[test]
    fn test_iteration_forward() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Test forward iteration
        let mut collected_keys = Vec::new();
        for (key, _value) in map.iter_forward() {
            collected_keys.push(key);
        }

        // Should be in ascending order
        let expected_keys = vec![10, 20, 30, 40, 50, 60, 70];
        assert_eq!(collected_keys, expected_keys);

        // Test using IntoIterator (default forward iteration)
        let mut collected_keys_2 = Vec::new();
        for (key, _value) in &map {
            collected_keys_2.push(key);
        }
        assert_eq!(collected_keys_2, expected_keys);
    }

    #[test]
    fn test_iteration_reverse() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Test reverse iteration
        let mut collected_keys = Vec::new();
        for (key, _value) in map.iter_reverse() {
            collected_keys.push(key);
        }

        // Should be in descending order
        let expected_keys = vec![70, 60, 50, 40, 30, 20, 10];
        assert_eq!(collected_keys, expected_keys);
    }

    #[test]
    fn test_iteration_with_direction() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert values
        for i in [3, 1, 4, 1, 5, 9, 2, 6] {
            map.insert(i, format!("pi_{i}"));
        }

        // Test forward direction
        let forward_keys: Vec<i32> = map
            .iter_with_direction(IterationDirection::Forward)
            .map(|(k, _)| k)
            .collect();
        assert_eq!(forward_keys, vec![1, 2, 3, 4, 5, 6, 9]);

        // Test reverse direction
        let reverse_keys: Vec<i32> = map
            .iter_with_direction(IterationDirection::Reverse)
            .map(|(k, _)| k)
            .collect();
        assert_eq!(reverse_keys, vec![9, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_iteration_empty_map() {
        let map: ShardedConcurrentBTreeMap<i32, String> = ShardedConcurrentBTreeMap::new();

        // Test forward iteration on empty map
        let forward_items: Vec<(i32, String)> = map.iter_forward().collect();
        assert_eq!(forward_items.len(), 0);

        // Test reverse iteration on empty map
        let reverse_items: Vec<(i32, String)> = map.iter_reverse().collect();
        assert_eq!(reverse_items.len(), 0);

        // Test IntoIterator on empty map
        let items: Vec<(i32, String)> = (&map).into_iter().collect();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_iteration_single_element() {
        let map = ShardedConcurrentBTreeMap::new();
        map.insert(42, "single".to_string());

        // Test forward iteration
        let forward_items: Vec<(i32, String)> = map.iter_forward().collect();
        assert_eq!(forward_items, vec![(42, "single".to_string())]);

        // Test reverse iteration
        let reverse_items: Vec<(i32, String)> = map.iter_reverse().collect();
        assert_eq!(reverse_items, vec![(42, "single".to_string())]);
    }

    #[test]
    fn test_iteration_values() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert key-value pairs where value is key * 10
        for i in [5, 2, 8, 1, 9] {
            map.insert(i, i * 10);
        }

        // Test that values are returned correctly in forward order
        let forward_pairs: Vec<(i32, i32)> = map.iter_forward().collect();
        assert_eq!(
            forward_pairs,
            vec![(1, 10), (2, 20), (5, 50), (8, 80), (9, 90)]
        );

        // Test that values are returned correctly in reverse order
        let reverse_pairs: Vec<(i32, i32)> = map.iter_reverse().collect();
        assert_eq!(
            reverse_pairs,
            vec![(9, 90), (8, 80), (5, 50), (2, 20), (1, 10)]
        );
    }

    #[test]
    fn test_iterator_size_hint() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert some values
        for i in 1..=5 {
            map.insert(i, format!("value_{i}"));
        }

        // Test size hint for forward iterator
        let forward_iter = map.iter_forward();
        let (lower, upper) = forward_iter.size_hint();
        assert_eq!(lower, 5);
        assert_eq!(upper, Some(5));

        // Test size hint for reverse iterator
        let reverse_iter = map.iter_reverse();
        let (lower, upper) = reverse_iter.size_hint();
        assert_eq!(lower, 5);
        assert_eq!(upper, Some(5));
    }

    #[test]
    fn test_keys_direction_methods() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert values in random order
        let values = vec![50, 10, 30, 70, 20, 60, 40];
        for &val in &values {
            map.insert(val, format!("value_{val}"));
        }

        // Test keys_forward (should be same as keys)
        let forward_keys = map.keys_forward();
        let regular_keys = map.keys();
        assert_eq!(forward_keys, regular_keys);
        assert_eq!(forward_keys, vec![10, 20, 30, 40, 50, 60, 70]);

        // Test keys_reverse
        let reverse_keys = map.keys_reverse();
        assert_eq!(reverse_keys, vec![70, 60, 50, 40, 30, 20, 10]);

        // Test keys_with_direction - forward
        let forward_direction_keys = map.keys_with_direction(IterationDirection::Forward);
        assert_eq!(forward_direction_keys, vec![10, 20, 30, 40, 50, 60, 70]);

        // Test keys_with_direction - reverse
        let reverse_direction_keys = map.keys_with_direction(IterationDirection::Reverse);
        assert_eq!(reverse_direction_keys, vec![70, 60, 50, 40, 30, 20, 10]);
    }

    #[test]
    fn test_keys_direction_empty_map() {
        let map: ShardedConcurrentBTreeMap<i32, String> = ShardedConcurrentBTreeMap::new();

        // Test all keys methods on empty map
        assert_eq!(map.keys().len(), 0);
        assert_eq!(map.keys_forward().len(), 0);
        assert_eq!(map.keys_reverse().len(), 0);
        assert_eq!(
            map.keys_with_direction(IterationDirection::Forward).len(),
            0
        );
        assert_eq!(
            map.keys_with_direction(IterationDirection::Reverse).len(),
            0
        );
    }

    #[test]
    fn test_keys_direction_single_element() {
        let map = ShardedConcurrentBTreeMap::new();
        map.insert(42, "answer".to_string());

        // All methods should return the same single key
        let expected = vec![42];
        assert_eq!(map.keys(), expected);
        assert_eq!(map.keys_forward(), expected);
        assert_eq!(map.keys_reverse(), expected);
        assert_eq!(
            map.keys_with_direction(IterationDirection::Forward),
            expected
        );
        assert_eq!(
            map.keys_with_direction(IterationDirection::Reverse),
            expected
        );
    }

    #[test]
    fn test_keys_direction_with_string_keys() {
        let map = ShardedConcurrentBTreeMap::new();

        // Insert string keys
        let keys = vec!["zebra", "apple", "banana", "cherry", "date"];
        for &key in &keys {
            map.insert(key.to_string(), format!("value_{key}"));
        }

        // Test forward order (alphabetical)
        let forward_keys = map.keys_forward();
        assert_eq!(
            forward_keys,
            vec!["apple", "banana", "cherry", "date", "zebra"]
        );

        // Test reverse order (reverse alphabetical)
        let reverse_keys = map.keys_reverse();
        assert_eq!(
            reverse_keys,
            vec!["zebra", "date", "cherry", "banana", "apple"]
        );
    }
}
