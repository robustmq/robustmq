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

use metadata_struct::connector::config_greptimedb::GreptimeDBConnectorConfig;
use metadata_struct::storage::record::StorageRecord;
use reqwest::header::{self, AUTHORIZATION};
use reqwest::Client;

use common_base::error::common::CommonError;

#[derive(Clone)]
pub struct Sender {
    client: Client,
    url: String,
}

impl Sender {
    #[allow(clippy::result_large_err)]
    pub fn new(config: &GreptimeDBConnectorConfig) -> Result<Self, CommonError> {
        let mut auth_header = header::HeaderMap::new();
        let token_value = format!("token {}:{}", config.user, config.password)
            .parse()
            .map_err(|e| CommonError::CommonError(format!("Invalid auth token format: {}", e)))?;
        auth_header.insert(AUTHORIZATION, token_value);

        let builder = Client::builder()
            .default_headers(auth_header)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30));

        let client = builder
            .build()
            .map_err(|e| CommonError::CommonError(format!("Failed to build HTTP client: {}", e)))?;

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

    #[allow(clippy::result_large_err)]
    fn record_to_line(record: &StorageRecord) -> Result<String, CommonError> {
        let mut tags = Vec::new();
        if let Some(headers) = &record.metadata.header {
            tags.reserve(headers.len());
            for header in headers {
                let tag_key = Self::escape_tag_value(&header.name);
                let tag_value = Self::escape_tag_value(&header.value);
                tags.push(format!("{}={}", tag_key, tag_value));
            }
        }
        let tags = tags.join(",");

        let mut fields = Vec::with_capacity(4);
        fields.push(format!("pkid={}i", record.metadata.create_t));

        let data_json = serde_json::to_string(&record.data).map_err(|e| {
            CommonError::CommonError(format!("Failed to serialize record data: {}", e))
        })?;
        let escaped_data = Self::escape_field_value(&data_json);
        fields.push(format!(r#"data="{}""#, escaped_data));

        let tags_json = serde_json::to_string(&record.metadata.tags).map_err(|e| {
            CommonError::CommonError(format!("Failed to serialize record tags: {}", e))
        })?;
        let escaped_tags = Self::escape_field_value(&tags_json);
        fields.push(format!(r#"tags="{}""#, escaped_tags));

        let fields = fields.join(",");

        let key_str = record
            .metadata
            .key
            .as_deref()
            .map(|k| String::from_utf8_lossy(k).into_owned());
        let measurement = Self::escape_measurement(key_str.as_deref().unwrap_or("unknown"));
        Ok(format!(
            "{},{} {} {}",
            measurement, tags, fields, record.metadata.create_t
        ))
    }

    pub async fn send(&self, data: &StorageRecord) -> Result<(), CommonError> {
        let line = Self::record_to_line(data)?;
        let res = self.client.post(&self.url).body(line).send().await?;

        let status = res.status();
        if !status.is_success() {
            let body = res
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(CommonError::CommonError(format!(
                "Failed to send to GreptimeDB: HTTP {}, response: {}",
                status.as_u16(),
                body
            )));
        }

        Ok(())
    }

    pub async fn send_batch(&self, records: &[StorageRecord]) -> Result<(), CommonError> {
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
            return Err(CommonError::CommonError(format!(
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
    fn record_to_line_includes_escaped_tags_fields_and_timestamp() {
        let record = StorageRecord {
            metadata: metadata_struct::storage::record::StorageRecordMetadata {
                header: Some(vec![metadata_struct::storage::record::StorageHeader {
                    name: "tag key".to_string(),
                    value: "tag,value".to_string(),
                }]),
                key: Some(bytes::Bytes::from_static(b"measurement name")),
                tags: Some(vec!["tag\"value".to_string()]),
                create_t: 42,
                ..Default::default()
            },
            protocol_data: None,
            data: bytes::Bytes::from_static(b"payload"),
        };

        let line = Sender::record_to_line(&record).unwrap();

        assert_eq!(
            line,
            "measurement\\ name,tag\\ key=tag\\,value pkid=42i,data=\"[112,97,121,108,111,97,100]\",tags=\"[\\\"tag\\\\\\\"value\\\"]\" 42"
        );
    }

    #[test]
    fn new_builds_url_with_precision() {
        let config = GreptimeDBConnectorConfig::new(
            "localhost:4000".to_string(),
            "public".to_string(),
            "user".to_string(),
            "password".to_string(),
        );

        let sender = Sender::new(&config).unwrap();

        assert_eq!(
            sender.url,
            "http://localhost:4000/v1/influxdb/api/v2/write?db=public&precision=s"
        );
    }
}
