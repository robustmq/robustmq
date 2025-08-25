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
    pub fn new(config: &GreptimeDBConnectorConfig) -> Self {
        let mut auth_header = header::HeaderMap::new();
        auth_header.insert(
            AUTHORIZATION,
            format!("token {}:{}", config.user, config.password)
                .parse()
                .unwrap(),
        );
        let builder = Client::builder().default_headers(auth_header);
        Self {
            client: builder.build().unwrap(),
            url: Self::build_url(config),
        }
    }

    fn build_url(config: &GreptimeDBConnectorConfig) -> String {
        format!(
            "http://{}/v1/influxdb/api/v2/write?db={}&precision={}",
            config.server_addr, config.database, config.precision
        )
    }
    fn record_to_line(record: &Record) -> String {
        // transform `Header` into influxDB line protocol tags
        let mut tags = Vec::with_capacity(record.header.len());
        for header in &record.header {
            tags.push(format!("{}={}", header.name, header.value));
        }
        let tags = tags.join(",");
        let mut fields = Vec::with_capacity(record.tags.len());
        if let Some(offset) = record.offset {
            fields.push(format!("offset={offset}i"));
        }
        fields.push(format!(
            r#"data="{}""#,
            serde_json::to_string(&record.data).unwrap()
        ));
        fields.push(format!(
            r#"tags="{}""#,
            serde_json::to_string(&record.data).unwrap()
        ));
        fields.push(format!("crc_num={}i", &record.crc_num));
        let fields = fields.join(",");

        format!("{},{} {} {}", record.key, tags, fields, record.timestamp)
    }

    pub async fn send(&self, data: &Record) -> ResultMqttBrokerError {
        let res = self
            .client
            .post(&self.url)
            .body(Self::record_to_line(data))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(MqttBrokerError::CommonError(
                "send to greptimedb failed, http status is not 200".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::adapter::record::Header;

    #[tokio::test]
    async fn test_send() {
        // Before test, run the following command to use docker setup a greptimedb server
        //
        // docker run -p 4000-4004:4000-4004 \
        // -p 4242:4242 -v "$(pwd)/greptimedb:/tmp/greptimedb" \
        // --name greptime --rm \
        // greptime/greptimedb standalone start \
        // --http-addr 0.0.0.0:4000 \
        // --rpc-addr 0.0.0.0:4001 \
        // --mysql-addr 0.0.0.0:4002 \
        // --user-provider=static_user_provider:cmd:greptime_user=greptime_pwd
        let config = GreptimeDBConnectorConfig::new(
            "127.0.0.1:4000".to_string(),
            "public".to_string(),
            "greptime_user".to_string(),
            "greptime_pwd".to_string(),
        );
        let sender = Sender::new(&config);
        let mut record = Record::build_str("test".to_string());
        record.set_key("test".to_string());
        record.set_header(vec![Header {
            name: "h1".to_string(),
            value: "v1".to_string(),
        }]);
        record.set_tags(vec!["t1".to_string(), "t2".to_string()]);
        let _ = sender.send(&record).await;
    }
}
