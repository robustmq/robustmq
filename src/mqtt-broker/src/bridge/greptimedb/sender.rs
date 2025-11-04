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

use std::time::Duration;

use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::bridge::config_greptimedb::GreptimeDBConnectorConfig;
use reqwest::header::{self, AUTHORIZATION};
use reqwest::Client;

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;

#[derive(Clone)]
pub struct Sender {
    client: Client,
    url: String,
}

impl Sender {
    pub fn new(config: &GreptimeDBConnectorConfig) -> Result<Self, MqttBrokerError> {
        let mut auth_header = header::HeaderMap::new();
        let token_value = format!("token {}:{}", config.user, config.password)
            .parse()
            .map_err(|e| {
                MqttBrokerError::CommonError(format!("Invalid auth token format: {}", e))
            })?;
        auth_header.insert(AUTHORIZATION, token_value);

        let builder = Client::builder()
            .default_headers(auth_header)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30));

        let client = builder.build().map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to build HTTP client: {}", e))
        })?;

        Ok(Self {
            client,
            url: Self::build_url(config),
        })
    }

    fn build_url(config: &GreptimeDBConnectorConfig) -> String {
        format!(
            "http://{}/v1/influxdb/api/v2/write?db={}&precision={}",
            config.server_addr, config.database, config.precision
        )
    }

    fn escape_tag_value(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace(',', "\\,")
            .replace('=', "\\=")
            .replace(' ', "\\ ")
    }

    fn escape_field_value(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn escape_measurement(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace(',', "\\,")
            .replace(' ', "\\ ")
    }

    fn record_to_line(record: &Record) -> Result<String, MqttBrokerError> {
        let mut tags = Vec::new();
        if let Some(headers) = &record.header {
            tags.reserve(headers.len());
            for header in headers {
                let tag_key = Self::escape_tag_value(&header.name);
                let tag_value = Self::escape_tag_value(&header.value);
                tags.push(format!("{}={}", tag_key, tag_value));
            }
        }
        let tags = tags.join(",");

        let mut fields = Vec::with_capacity(4);
        if let Some(offset) = record.offset {
            fields.push(format!("offset={offset}i"));
        }

        let data_json = serde_json::to_string(&record.data).map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to serialize record data: {}", e))
        })?;
        let escaped_data = Self::escape_field_value(&data_json);
        fields.push(format!(r#"data="{}""#, escaped_data));

        let tags_json = serde_json::to_string(&record.tags).map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to serialize record tags: {}", e))
        })?;
        let escaped_tags = Self::escape_field_value(&tags_json);
        fields.push(format!(r#"tags="{}""#, escaped_tags));

        fields.push(format!("crc_num={}i", record.crc_num));
        let fields = fields.join(",");

        let measurement = Self::escape_measurement(record.key.as_deref().unwrap_or("unknown"));
        Ok(format!(
            "{},{} {} {}",
            measurement, tags, fields, record.timestamp
        ))
    }

    pub async fn send(&self, data: &Record) -> ResultMqttBrokerError {
        let line = Self::record_to_line(data)?;
        let res = self.client.post(&self.url).body(line).send().await?;

        let status = res.status();
        if !status.is_success() {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(MqttBrokerError::CommonError(format!(
                "Failed to send to GreptimeDB: HTTP {}, response: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(())
    }

    pub async fn send_batch(&self, records: &[Record]) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        let mut lines = Vec::with_capacity(records.len());
        for record in records {
            lines.push(Self::record_to_line(record)?);
        }
        let body = lines.join("\n");

        let res = self.client.post(&self.url).body(body).send().await?;

        let status = res.status();
        if !status.is_success() {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(MqttBrokerError::CommonError(format!(
                "Failed to send batch to GreptimeDB: HTTP {}, response: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::adapter::record::Header;

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_send() {
        let config = GreptimeDBConnectorConfig::new(
            "127.0.0.1:4000".to_string(),
            "public".to_string(),
            "greptime_user".to_string(),
            "greptime_pwd".to_string(),
        );
        let sender = Sender::new(&config).expect("Failed to create sender");
        let mut record = Record::from_string("test".to_string());
        record.set_key("test".to_string());
        record.set_header(vec![Header {
            name: "h1".to_string(),
            value: "v1".to_string(),
        }]);
        record.set_tags(vec!["t1".to_string(), "t2".to_string()]);
        let result = sender.send(&record).await;
        assert!(result.is_ok(), "Send should succeed: {:?}", result);
    }

    #[test]
    fn test_escape_functions() {
        assert_eq!(Sender::escape_tag_value("hello world"), "hello\\ world");
        assert_eq!(Sender::escape_tag_value("key=value"), "key\\=value");
        assert_eq!(Sender::escape_tag_value("a,b,c"), "a\\,b\\,c");
        assert_eq!(Sender::escape_tag_value("path\\file"), "path\\\\file");

        assert_eq!(Sender::escape_field_value("hello\"world"), "hello\\\"world");
        assert_eq!(Sender::escape_field_value("path\\file"), "path\\\\file");

        assert_eq!(
            Sender::escape_measurement("my measurement"),
            "my\\ measurement"
        );
        assert_eq!(
            Sender::escape_measurement("my,measurement"),
            "my\\,measurement"
        );
    }

    #[test]
    fn test_record_to_line() {
        let mut record = Record::from_string("test data".to_string());
        record.set_key("sensor_data".to_string());
        record.set_header(vec![Header {
            name: "location".to_string(),
            value: "room 1".to_string(),
        }]);
        record.set_tags(vec!["tag1".to_string()]);
        record.timestamp = 1234567890;

        let line = Sender::record_to_line(&record).expect("Should convert to line");
        assert!(line.contains("sensor_data"));
        assert!(line.contains("location=room\\ 1"));
        assert!(line.contains("1234567890"));
    }
}
