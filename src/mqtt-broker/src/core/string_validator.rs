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

/// Common UTF-8 string validation for MQTT protocol
///
/// MQTT 5.0 UTF-8 String Requirements:
/// - Must be valid UTF-8
/// - Must NOT contain null character (U+0000)
/// - Must NOT contain control characters (except tab, newline, carriage return)
/// - Length must not exceed 65535 bytes

/// Validates a UTF-8 string according to MQTT 5.0 specification
pub fn validate_utf8_string(
    value: &str,
    field_name: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), String> {
    if value.len() < min_len {
        return Err(format!(
            "{} is too short (min: {} bytes, got: {} bytes)",
            field_name,
            min_len,
            value.len()
        ));
    }

    if value.len() > max_len {
        return Err(format!(
            "{} exceeds maximum length (max: {} bytes, got: {} bytes)",
            field_name,
            max_len,
            value.len()
        ));
    }

    if value.contains('\0') {
        return Err(format!("{} contains null character (U+0000)", field_name));
    }

    for c in value.chars() {
        if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
            return Err(format!("{} contains invalid control character", field_name));
        }
    }

    Ok(())
}

/// Validates MQTT Client Identifier
///
/// MQTT 3.1.1: 1-23 characters
/// MQTT 5.0: 1-65535 characters
pub fn validate_client_id(client_id: &str, is_mqtt5: bool) -> Result<(), String> {
    if client_id.is_empty() {
        return Ok(());
    }

    let max_len = if is_mqtt5 { 65535 } else { 23 };
    validate_utf8_string(client_id, "Client ID", 1, max_len)
}

/// Validates MQTT Username
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    validate_utf8_string(username, "Username", 1, 65535)
}

/// Validates MQTT Password
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }
    validate_utf8_string(password, "Password", 1, 65535)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_utf8_string() {
        assert!(validate_utf8_string("test", "field", 1, 100).is_ok());
        assert!(validate_utf8_string("", "field", 0, 100).is_ok());
        assert!(validate_utf8_string("", "field", 1, 100).is_err());
        assert!(validate_utf8_string("too_long", "field", 1, 5).is_err());
    }

    #[test]
    fn test_validate_utf8_string_null_char() {
        let with_null = "test\0value";
        assert!(validate_utf8_string(with_null, "field", 1, 100).is_err());
    }

    #[test]
    fn test_validate_utf8_string_control_chars() {
        assert!(validate_utf8_string("test\ttab", "field", 1, 100).is_ok());
        assert!(validate_utf8_string("test\nnewline", "field", 1, 100).is_ok());
        assert!(validate_utf8_string("test\rcarriage", "field", 1, 100).is_ok());

        let with_control = "test\x01value";
        assert!(validate_utf8_string(with_control, "field", 1, 100).is_err());
    }

    #[test]
    fn test_validate_client_id_mqtt311() {
        assert!(validate_client_id("test", false).is_ok());
        assert!(validate_client_id("", false).is_ok());
        assert!(validate_client_id(&"a".repeat(23), false).is_ok());
        assert!(validate_client_id(&"a".repeat(24), false).is_err());
    }

    #[test]
    fn test_validate_client_id_mqtt5() {
        assert!(validate_client_id("test", true).is_ok());
        assert!(validate_client_id("", true).is_ok());
        assert!(validate_client_id(&"a".repeat(100), true).is_ok());
        assert!(validate_client_id(&"a".repeat(65536), true).is_err());
    }

    #[test]
    fn test_validate_client_id_with_null() {
        assert!(validate_client_id("test\0client", false).is_err());
        assert!(validate_client_id("test\0client", true).is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("user\0name").is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("pass123").is_ok());
        assert!(validate_password("").is_err());
        assert!(validate_password("pass\0word").is_err());
    }

    #[test]
    fn test_validate_utf8_emoji() {
        assert!(validate_client_id("client_🚀", true).is_ok());
        assert!(validate_username("user_😀").is_ok());
        assert!(validate_password("pass_🔒").is_ok());
    }
}
