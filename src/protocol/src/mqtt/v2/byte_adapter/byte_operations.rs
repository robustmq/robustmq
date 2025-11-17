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

use bytes::{Buf, BufMut, BytesMut};

#[allow(dead_code)]
pub(crate) trait ByteOperations {
    fn read_a_byte(&mut self) -> Option<u8>;
    fn read_bytes(&mut self, len: usize) -> Vec<u8>;
    fn write_a_byte(&mut self, byte: u8);
    fn write_bytes(&mut self, bytes: &[u8]);
    fn bytes_len(&self) -> usize;
    fn available_size(&mut self, want_size: usize) -> usize {
        want_size.min(self.bytes_len())
    }
    fn is_empty(&self) -> bool {
        self.bytes_len() == 0
    }
}

impl ByteOperations for BytesMut {
    fn read_a_byte(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        Some(self.get_u8())
    }

    fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let take_size = self.available_size(len);
        self.split_to(take_size).to_vec()
    }

    fn write_a_byte(&mut self, byte: u8) {
        self.put_u8(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn bytes_len(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod byte_ops_tests {
    use crate::mqtt::v2::byte_adapter::byte_operations::ByteOperations;
    use bytes::BytesMut;

    #[test]
    fn bytes_mut_can_write_a_byte() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_a_byte(0xAB);
        assert_eq!(bytes_mut.bytes_len(), 1);
    }

    #[test]
    fn bytes_mut_can_write_bytes() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0x01, 0x02, 0x03]);
        assert_eq!(bytes_mut.bytes_len(), 3);
    }

    #[test]
    fn bytes_mut_can_read_a_byte() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_a_byte(0xCD);
        assert_eq!(bytes_mut.bytes_len(), 1);
        let byte = bytes_mut.read_a_byte();
        assert_eq!(byte, Some(0xCD));
    }

    #[test]
    fn bytes_must_can_read_bytes() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0x01, 0x02, 0x03]);
        assert_eq!(bytes_mut.bytes_len(), 3);
        let bytes = bytes_mut.read_bytes(3);
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn bytes_mut_bytes_len_should_return_correct_length() {
        let mut bytes_mut = BytesMut::new();
        assert_eq!(bytes_mut.bytes_len(), 0);
        bytes_mut.write_bytes(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(bytes_mut.bytes_len(), 4);
    }

    #[test]
    fn bytes_mut_read_more_than_available_bytes() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0x01, 0x02]);
        assert_eq!(bytes_mut.bytes_len(), 2);
        let bytes = bytes_mut.read_bytes(3);
        assert_eq!(bytes, vec![0x01, 0x02]);
    }

    #[test]
    fn bytes_mut_read_a_byte_from_empty_bytes_mut_should_return_none() {
        let mut bytes_mut = BytesMut::new();
        let byte = bytes_mut.read_a_byte();
        assert!(byte.is_none());
        assert_eq!(byte, None);
    }

    #[test]
    fn byte_mut_read_bytes_from_empty_bytes_mut_should_return_empty_vec() {
        let mut bytes_mut = BytesMut::new();
        let bytes = bytes_mut.read_bytes(5);
        assert!(bytes.is_empty());
    }

    #[test]
    fn bytes_mut_get_available_len_should_return_correct_length() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_bytes(&[0x01, 0x02, 0x03]);
        assert_eq!(bytes_mut.available_size(2), 2);
        assert_eq!(bytes_mut.available_size(5), 3);
    }
}
