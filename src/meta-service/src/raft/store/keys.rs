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

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use common_base::error::common::CommonError;
use std::io::Cursor;

// Key encoding using Big Endian to ensure lexicographic order equals numeric order.
// State Keys: [prefix:1][machine_hash:8][state_type:1] = 10 bytes
// Log Keys:   [prefix:1][machine_hash:8][log_index:8]  = 17 bytes

const STATE_PREFIX: u8 = b'S';
const STATE_LAST_PURGED: u8 = 1;
const STATE_COMMITTED: u8 = 2;
const STATE_VOTE: u8 = 3;
const LOG_PREFIX: u8 = b'L';

/// Hash machine name using FNV-1a algorithm
fn hash_machine_name(machine: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in machine.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn make_state_key(machine: &str, state_type: u8) -> Vec<u8> {
    let machine_hash = hash_machine_name(machine);
    let mut key = Vec::with_capacity(10);
    key.push(STATE_PREFIX);
    key.write_u64::<BigEndian>(machine_hash).unwrap();
    key.push(state_type);
    key
}

pub fn key_last_purged_log_id(machine: &str) -> Vec<u8> {
    make_state_key(machine, STATE_LAST_PURGED)
}

pub fn key_committed(machine: &str) -> Vec<u8> {
    make_state_key(machine, STATE_COMMITTED)
}

pub fn key_vote(machine: &str) -> Vec<u8> {
    make_state_key(machine, STATE_VOTE)
}

/// Generate Raft log key: [L:1][machine_hash:8][log_index:8 Big Endian]
/// Big Endian ensures correct sorting: 9 < 10 < 100
pub fn key_raft_log(machine: &str, index: u64) -> Vec<u8> {
    let machine_hash = hash_machine_name(machine);
    let mut key = Vec::with_capacity(17);
    key.push(LOG_PREFIX);
    key.write_u64::<BigEndian>(machine_hash).unwrap();
    key.write_u64::<BigEndian>(index).unwrap();
    key
}

/// Extract log index from Raft log key
pub fn raft_log_key_to_id(machine: &str, key: &[u8]) -> Result<u64, CommonError> {
    if key.len() < 17 {
        return Err(CommonError::CommonError(format!(
            "Key too short: expected 17 bytes, got {}",
            key.len()
        )));
    }

    if key[0] != LOG_PREFIX {
        return Err(CommonError::CommonError(format!(
            "Invalid log key prefix: expected 'L', got {}",
            key[0] as char
        )));
    }

    let expected_hash = hash_machine_name(machine);
    let mut cursor = Cursor::new(&key[1..9]);
    let key_hash = cursor
        .read_u64::<BigEndian>()
        .map_err(|e| CommonError::CommonError(format!("Failed to read machine hash: {}", e)))?;

    if key_hash != expected_hash {
        return Err(CommonError::CommonError(format!(
            "Machine hash mismatch: expected 0x{:016x}, got 0x{:016x}",
            expected_hash, key_hash
        )));
    }

    let mut cursor = Cursor::new(&key[9..17]);
    cursor
        .read_u64::<BigEndian>()
        .map_err(|e| CommonError::CommonError(format!("Failed to read log index: {}", e)))
}

pub fn raft_log_range(machine: &str) -> (Vec<u8>, Vec<u8>) {
    (key_raft_log(machine, 0), key_raft_log(machine, u64::MAX))
}

pub fn raft_log_range_from_to(
    machine: &str,
    start_index: u64,
    end_index: u64,
) -> (Vec<u8>, Vec<u8>) {
    (
        key_raft_log(machine, start_index),
        key_raft_log(machine, end_index),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_log_key_sorting() {
        let machine = "test_machine";
        let indices = vec![9, 10, 100, 1000, 99, 101, 1, 5];
        let mut keys: Vec<(u64, Vec<u8>)> = indices
            .iter()
            .map(|&idx| (idx, key_raft_log(machine, idx)))
            .collect();

        keys.sort_by(|a, b| a.1.cmp(&b.1));

        let sorted_indices: Vec<u64> = keys.iter().map(|(idx, _)| *idx).collect();
        let mut expected = indices.clone();
        expected.sort();

        assert_eq!(sorted_indices, expected);
    }

    #[test]
    fn test_key_codec() {
        let machine = "test_machine";

        for index in [0, 1, 100, 1000, u64::MAX] {
            let key = key_raft_log(machine, index);
            assert_eq!(key.len(), 17);
            assert_eq!(raft_log_key_to_id(machine, &key).unwrap(), index);
        }

        let state_keys = vec![
            key_last_purged_log_id(machine),
            key_committed(machine),
            key_vote(machine),
        ];

        for key in state_keys {
            assert_eq!(key.len(), 10);
            assert_eq!(key[0], b'S');
        }
    }

    #[test]
    fn test_key_validation() {
        let key = key_raft_log("machine1", 100);
        assert!(raft_log_key_to_id("machine2", &key).is_err());
        assert!(raft_log_key_to_id("test", b"invalid").is_err());
    }

    #[test]
    fn test_range_queries() {
        let machine = "test_machine";
        let (start, end) = raft_log_range(machine);
        assert!(start < end);

        for index in [0, 1000, u64::MAX] {
            let key = key_raft_log(machine, index);
            assert!(key >= start && key <= end);
        }
    }
}
