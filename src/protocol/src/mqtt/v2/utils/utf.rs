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
pub(crate) mod utf_8_handler {

    use crate::mqtt::v2::byte_adapter::byte_operations::ByteOperations;
    use crate::mqtt::v2::utils::code_error::CodeError;
    use crate::mqtt::v2::utils::radix::radix_handler;
    use std::ops::RangeInclusive;

    pub fn parse(byte_opts: &mut impl ByteOperations) -> Result<String, CodeError> {
        let utf_8_length = calculate_str_length(byte_opts)?;

        let utf8_string = read_str(byte_opts, utf_8_length)?;

        Ok(utf8_string)
    }

    pub(super) fn calculate_str_length(
        byte_opts: &mut impl ByteOperations,
    ) -> Result<u16, CodeError> {
        let length_bytes = byte_opts.read_bytes(2);
        let utf_8_length = radix_handler::be_bytes_to_u16(length_bytes.as_slice())?;
        Ok(utf_8_length)
    }

    pub(super) fn read_str(
        byte_opts: &mut impl ByteOperations,
        utf_8_length: u16,
    ) -> Result<String, CodeError> {
        let string_bytes = byte_opts.read_bytes(utf_8_length as usize);

        verify_for_mqtt(&string_bytes)?;

        let utf8_string = decode_utf8(string_bytes)?;
        Ok(utf8_string)
    }

    /// we don't verify 0xD800..=0xDFFF, because rust string already do that
    pub(super) fn verify_for_mqtt(string_bytes: &[u8]) -> Result<(), CodeError> {
        let str = std::str::from_utf8(string_bytes).map_err(|_| CodeError::UTF8DecodingError)?;
        const FORBIDDEN_CHAR_FOR_MQTT: &[RangeInclusive<u32>] = &[
            0x0000..=0x0000, // null
            0x0001..=0x001F, // C0
            0x007F..=0x009F, // C1
            0xFDD0..=0xFDEF, //
            0xFFFE..=0xFFFF, //
        ];

        for char in str.chars() {
            let value = char as u32;
            if value == 0xFEFF {
                continue;
            }
            if FORBIDDEN_CHAR_FOR_MQTT.iter().any(|r| r.contains(&value)) {
                return Err(CodeError::MQTTInvalidCode(value));
            }
        }

        Ok(())
    }

    pub(super) fn decode_utf8(string_bytes: Vec<u8>) -> Result<String, CodeError> {
        let utf8_string =
            String::from_utf8(string_bytes).map_err(|_| CodeError::UTF8DecodingError)?;
        Ok(utf8_string)
    }

    pub(super) fn encode_utf8(input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }

    pub(crate) fn write_utf8_for_mqtt(
        byte_opts: &mut impl ByteOperations,
        input: &str,
    ) -> Result<(), CodeError> {
        let string_bytes = encode_utf8(input);
        verify_for_mqtt(&string_bytes)?;
        let length_bytes = radix_handler::u16_to_be_2_bytes(string_bytes.len())?;
        byte_opts.write_bytes(&length_bytes);
        byte_opts.write_bytes(&string_bytes);
        Ok(())
    }
}

#[cfg(test)]
mod utf_8_tests {
    use crate::mqtt::v2::byte_adapter::byte_operations::ByteOperations;
    use crate::mqtt::v2::utils::code_error::CodeError;
    use crate::mqtt::v2::utils::utf::utf_8_handler;
    use bytes::BytesMut;

    #[test]
    fn utf_8_handler_should_read_2_byte_to_calculate_length() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.write_a_byte(0x00);
        bytes_mut.write_a_byte(0x05);
        let length = utf_8_handler::calculate_str_length(&mut bytes_mut).unwrap();
        assert_eq!(length, 5);
    }

    #[test]
    fn utf_8_handler_should_decode_utf_8_string() {
        let mut bytes_mut = BytesMut::new();
        let except_word = "hello";
        utf_8_handler::write_utf8_for_mqtt(&mut bytes_mut, except_word).unwrap();

        let utf8_string = utf_8_handler::parse(&mut bytes_mut).unwrap();

        assert_eq!(utf8_string, except_word);
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_d800() {
        let invalid_utf8_bytes = vec![0xED, 0xA0, 0x80];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::UTF8DecodingError)))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_dfff() {
        let invalid_utf8_bytes = vec![0xED, 0xBF, 0xBF];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::UTF8DecodingError)))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_0000() {
        let invalid_utf8_bytes = vec![0x00];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0x0000))))
    }
    #[test]
    fn utf_8_handler_should_not_allow_u_0001() {
        let invalid_utf8_bytes = vec![0x01];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0x0001))))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_001f() {
        let invalid_utf8_bytes = vec![0x1F];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0x001F))))
    }

    // todo decode utf-8 can not allowed U+007F to U+009F
    #[test]
    fn utf_8_handler_should_not_allow_u_007f() {
        let invalid_utf8_bytes = vec![0x7F];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0x007F))))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_009f() {
        let invalid_utf8_bytes = vec![0xC2, 0x9F];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0x009F))))
    }

    // todo decode utf-8 can not allowed U+FDD0 to U+FDEF
    #[test]
    fn utf_8_handler_should_not_allow_u_fdd0() {
        let invalid_utf8_bytes = vec![0xEF, 0xB7, 0x90];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0xFDD0))))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_fdef() {
        let invalid_utf8_bytes = vec![0xEF, 0xB7, 0xAF];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0xFDEF))))
    }

    #[test]
    fn utf_8_handler_should_not_allow_u_ffff() {
        let invalid_utf8_bytes = vec![0xEF, 0xBF, 0xBF];
        let result = utf_8_handler::verify_for_mqtt(&invalid_utf8_bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodeError::MQTTInvalidCode(0xFFFF))))
    }

    #[test]
    fn utf_8_handler_should_allow_u_feff() {
        let valid_utf8_bytes = vec![0xEF, 0xBB, 0xBF];
        let result = utf_8_handler::verify_for_mqtt(&valid_utf8_bytes);
        assert!(result.is_ok());
    }
}
