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

use metadata_struct::mqtt::bridge::config_pulsar::PulsarConnectorConfig;

use pulsar::{
    authentication::{basic::BasicAuthentication, oauth2::OAuth2Authentication},
    producer, Authentication, Error as PulsarError, Pulsar, TokioExecutor,
};

pub struct Producer<'a> {
    config: &'a PulsarConnectorConfig,
}

impl<'a> Producer<'a> {
    pub(crate) fn new(config: &'a PulsarConnectorConfig) -> Self {
        Producer { config }
    }

    pub(crate) async fn build_producer(
        &self,
    ) -> Result<producer::Producer<TokioExecutor>, PulsarError> {
        let builder = Pulsar::builder(&self.config.server, TokioExecutor);
        let builder = match (
            &self.config.token,
            &self.config.oauth,
            &self.config.basic_name,
            &self.config.basic_password,
        ) {
            (Some(token), None, None, None) => {
                let authentication = Authentication {
                    name: "token".to_string(),
                    data: token.to_owned().into_bytes(),
                };

                builder.with_auth(authentication)
            }
            (None, Some(oauth2_cfg), None, None) => {
                builder.with_auth_provider(OAuth2Authentication::client_credentials(
                    serde_json::from_str(oauth2_cfg.as_str()).unwrap_or_else(|_| {
                        panic!("invalid oauth2 config [{}]", oauth2_cfg.as_str())
                    }),
                ))
            }
            (None, None, Some(username), Some(password)) => {
                builder.with_auth_provider(BasicAuthentication::new(username, password))
            }
            _ => builder,
        };

        let pulsar = builder.build().await?;
        pulsar
            .producer()
            .with_topic(&self.config.topic)
            .build()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::adapter::record::Header;
    use metadata_struct::adapter::record::Record;

    // Test PulsarProducer works
    // Run `docker run --rm -it -p 6650:6650 -p 8080:8080 --name pulsar apachepulsar/pulsar:2.11.0 bin/pulsar standalone -nfw -nss` to set up pulsar instance, then run test.
    #[tokio::test]
    async fn test_send() {
        let config = PulsarConnectorConfig {
            server: "pulsar://localhost:6650".to_string(),
            topic: "my-topic".to_string(),
            token: None,
            oauth: None,
            basic_name: None,
            basic_password: None,
        };
        let producer = Producer::new(&config);
        let p = producer.build_producer().await;
        if let Ok(mut p) = p {
            let mut record = Record::build_str("test".to_string());
            record.set_key("test".to_string());
            record.set_header(vec![Header {
                name: "h1".to_string(),
                value: "v1".to_string(),
            }]);
            record.set_tags(vec!["t1".to_string(), "t2".to_string()]);

            p.send_non_blocking(record).await.unwrap();
        }
    }
}
