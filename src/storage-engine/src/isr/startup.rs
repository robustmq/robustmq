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

use crate::core::error::StorageEngineError;
use crate::isr::leader_epoch::LeaderEpochCache;

pub fn recover_leader_epoch_cache(
    cache: &mut LeaderEpochCache,
    local_leo: u64,
    log_start_offset: u64,
) -> Result<(), StorageEngineError> {
    cache.truncate_from_end(local_leo)?;
    cache.truncate_from_start(log_start_offset)?;
    Ok(())
}

pub fn recover_hw(persisted_hw: u64, local_leo: u64) -> u64 {
    persisted_hw.min(local_leo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isr::leader_epoch::LeaderEpochCache;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn cache() -> LeaderEpochCache {
        LeaderEpochCache::load(test_rocksdb_instance(), "s", 0).unwrap()
    }

    #[test]
    fn recover_trims_both_ends() {
        let mut c = cache();
        c.assign(1, 0).unwrap();
        c.assign(2, 5).unwrap();
        c.assign(3, 9).unwrap();

        recover_leader_epoch_cache(&mut c, 8, 3).unwrap();

        assert_eq!(c.latest_epoch(), 2);
        assert_eq!(c.end_offset_for(1), Some(5));
        assert_eq!(c.end_offset_for(2), None);
    }

    #[test]
    fn hw_clamped_to_leo() {
        assert_eq!(recover_hw(8, 5), 5);
        assert_eq!(recover_hw(3, 5), 3);
    }
}
