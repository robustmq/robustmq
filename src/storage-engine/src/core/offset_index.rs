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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentOffsetRange {
    pub segment_seq: u32,
    pub start_offset: i64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

#[derive(Clone, Default)]
pub struct SegmentOffsetIndex {
    ranges: Vec<SegmentOffsetRange>,
}

impl SegmentOffsetIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        segment_seq: u32,
        start_offset: i64,
        start_timestamp: i64,
        end_timestamp: i64,
    ) {
        if let Some(pos) = self
            .ranges
            .iter()
            .position(|r| r.segment_seq == segment_seq)
        {
            self.ranges[pos].start_offset = start_offset;
            self.ranges[pos].start_timestamp = start_timestamp;
            self.ranges[pos].end_timestamp = end_timestamp;
        } else {
            self.ranges.push(SegmentOffsetRange {
                segment_seq,
                start_offset,
                start_timestamp,
                end_timestamp,
            });
        }
    }

    pub fn delete(&mut self, segment_seq: u32) {
        self.ranges.retain(|r| r.segment_seq != segment_seq);
    }

    pub fn sort(&mut self) {
        self.ranges.sort_by_key(|r| r.start_offset);
    }

    pub fn find_segment(&self, offset: i64) -> Option<u32> {
        if self.ranges.is_empty() {
            return None;
        }

        let idx = self.ranges.partition_point(|r| r.start_offset <= offset);

        if idx > 0 {
            Some(self.ranges[idx - 1].segment_seq)
        } else {
            None
        }
    }

    pub fn find_segment_by_timestamp(&self, timestamp: i64) -> Option<u32> {
        self.ranges
            .iter()
            .filter(|r| r.start_timestamp <= timestamp && timestamp <= r.end_timestamp)
            .min_by_key(|r| r.start_offset)
            .map(|r| r.segment_seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_index() {
        let mut index = SegmentOffsetIndex::new();

        index.add(2, 2000, 3000, 3999);
        index.add(0, 0, 1000, 1999);
        index.add(1, 1000, 2000, 2999);
        index.add(0, 100, 1100, 1999);

        assert_eq!(index.ranges.len(), 3);

        index.sort();

        assert_eq!(index.find_segment(150), Some(0));
        assert_eq!(index.find_segment(1500), Some(1));
        assert_eq!(index.find_segment(2500), Some(2));

        assert_eq!(index.find_segment_by_timestamp(1500), Some(0));
        assert_eq!(index.find_segment_by_timestamp(2500), Some(1));
        assert_eq!(index.find_segment_by_timestamp(3500), Some(2));
        assert_eq!(index.find_segment_by_timestamp(5000), None);

        index.delete(1);

        assert_eq!(index.ranges.len(), 2);
        assert_eq!(index.find_segment(150), Some(0));
        assert_eq!(index.find_segment(2500), Some(2));
        assert_eq!(index.find_segment_by_timestamp(2500), None);
        assert_eq!(index.find_segment_by_timestamp(3500), Some(2));
    }
}
