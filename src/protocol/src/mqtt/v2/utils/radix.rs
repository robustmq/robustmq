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

#[allow(dead_code)]
pub(crate) mod radix_handler {
    use crate::mqtt::v2::utils::code_error::CodeError;

    #[inline]
    pub(crate) fn low_nibble(binary_byte: u8) -> u8 {
        binary_byte & 0x0F
    }

    #[inline]
    pub(crate) fn high_nibble(binary_byte: u8) -> u8 {
        binary_byte >> 4
    }

    #[inline]
    pub(crate) fn binary_byte_to_decimal(binary_byte: u8) -> u8 {
        binary_byte
    }
    #[inline]
    pub(crate) fn decimal_to_binary_byte(decimal_value: u8) -> u8 {
        decimal_value
    }

    #[inline]
    pub(crate) fn be_bytes_to_u16(bytes: &[u8]) -> Result<u16, CodeError> {
        bytes
            .get(..2)
            .and_then(|slice| slice.try_into().ok())
            .map(u16::from_be_bytes)
            .ok_or(CodeError::CodeLengthError(2, bytes.len()))
    }

    pub(crate) fn u16_to_be_2_bytes(length: usize) -> Result<[u8; 2], CodeError> {
        if length > u16::MAX as usize {
            return Err(CodeError::UsizeConversionError(length, "u16"));
        }
        let length_u16 = length as u16;
        Ok(length_u16.to_be_bytes())
    }
}

#[cfg(test)]
mod binary_util_test {
    use crate::mqtt::v2::utils::code_error::CodeError;
    use crate::mqtt::v2::utils::radix::radix_handler;

    #[test]
    fn binary_high_4bits_should_convert_to_8bits() {
        let value: u8 = 0b0000_0101;
        let binary_high_4bits = radix_handler::high_nibble(value);
        assert_eq!(binary_high_4bits, 0b0000_0000);
    }

    #[test]
    fn binary_low_4bits_should_convert_to_8bits() {
        let value: u8 = 0b1010_1111;
        let binary_low_4bits = radix_handler::low_nibble(value);
        assert_eq!(binary_low_4bits, 0b0000_1111);
    }

    #[test]
    fn binary_8bits_should_convert_to_decimal() {
        let value: u8 = 0b0000_1010;
        let value = radix_handler::binary_byte_to_decimal(value);
        assert_eq!(value, 10);
    }

    #[test]
    fn decimal_should_convert_to_binary_8bits() {
        let value: u8 = 10;
        let value = radix_handler::decimal_to_binary_byte(value);
        assert_eq!(value, 0b0000_1010);
    }

    #[test]
    fn be_bytes_should_convert_to_u16() {
        let bytes: Vec<u8> = vec![0x12, 0x34];
        let value = radix_handler::be_bytes_to_u16(&bytes).unwrap();
        assert_eq!(value, 0x1234);
    }

    #[test]
    fn be_bytes_to_u16_should_return_error_when_length_invalid() {
        let bytes: Vec<u8> = vec![0x12];
        let result = radix_handler::be_bytes_to_u16(&bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::CodeLengthError(2, 1))));
    }

    #[test]
    fn usize_can_not_exceed_u16_max_value() {
        let size: usize = 70000;
        let result = radix_handler::u16_to_be_2_bytes(size);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CodeError::UsizeConversionError(_size, "u16"))
        ));
    }

    #[test]
    fn usize_can_convert_to_be_2byte() {
        let size: usize = 50000;
        let bytes = radix_handler::u16_to_be_2_bytes(size).unwrap();
        assert_eq!(bytes, [0xC3, 0x50]);
    }

    #[test]
    fn usize_can_convert_to_be_2byte_min_value() {
        let size: usize = 0;
        let bytes = radix_handler::u16_to_be_2_bytes(size).unwrap();
        assert_eq!(bytes, [0x00, 0x00]);
    }

    #[test]
    fn usize_can_convert_to_be_2byte_max_value() {
        let size: usize = 65535;
        let bytes = radix_handler::u16_to_be_2_bytes(size).unwrap();
        assert_eq!(bytes, [0xFF, 0xFF]);
    }
}
