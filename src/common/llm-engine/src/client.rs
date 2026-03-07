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

use common_base::error::common::CommonError;
use common_config::config::{LLMClientConfig, LLMPlatform};
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{AuthData, Endpoint};
use genai::{Client, ModelIden, ServiceTarget};

fn platform_to_adapter_kind(platform: &LLMPlatform) -> AdapterKind {
    match platform {
        LLMPlatform::OpenAI => AdapterKind::OpenAI,
        LLMPlatform::OpenAIResp => AdapterKind::OpenAIResp,
        LLMPlatform::Gemini => AdapterKind::Gemini,
        LLMPlatform::Anthropic => AdapterKind::Anthropic,
        LLMPlatform::Fireworks => AdapterKind::Fireworks,
        LLMPlatform::Together => AdapterKind::Together,
        LLMPlatform::Groq => AdapterKind::Groq,
        LLMPlatform::Mimo => AdapterKind::Mimo,
        LLMPlatform::Nebius => AdapterKind::Nebius,
        LLMPlatform::Xai => AdapterKind::Xai,
        LLMPlatform::DeepSeek => AdapterKind::DeepSeek,
        LLMPlatform::Zai => AdapterKind::Zai,
        LLMPlatform::BigModel => AdapterKind::BigModel,
        LLMPlatform::Cohere => AdapterKind::Cohere,
        LLMPlatform::Ollama => AdapterKind::Ollama,
    }
}

pub struct LLMClient {
    model_name: String,
    client: Client,
}

impl LLMClient {
    pub fn new(config: LLMClientConfig) -> Result<Self, CommonError> {
        config.validate()?;

        let adapter_kind = platform_to_adapter_kind(&config.platform);
        let base_url = config.base_url.clone();
        let token = config.token.clone();

        let client = Client::builder()
            .with_auth_resolver_fn(move |_model_iden| Ok(token.clone().map(AuthData::from_single)))
            .with_service_target_resolver_fn(move |service_target: ServiceTarget| {
                let endpoint = match &base_url {
                    Some(url) => Endpoint::from_owned(url.clone()),
                    None => service_target.endpoint,
                };

                let model = ModelIden::new(adapter_kind, service_target.model.model_name);
                Ok(ServiceTarget {
                    endpoint,
                    auth: service_target.auth,
                    model,
                })
            })
            .build();

        let model_name = config.model.clone();
        Ok(Self { model_name, client })
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, CommonError> {
        self.chat_with_system(None, prompt).await
    }

    pub async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        prompt: &str,
    ) -> Result<String, CommonError> {
        if prompt.trim().is_empty() {
            return Err(CommonError::CommonError(
                "prompt cannot be empty".to_string(),
            ));
        }

        let mut chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);
        if let Some(system_prompt) = system_prompt {
            if !system_prompt.trim().is_empty() {
                chat_req = chat_req.with_system(system_prompt);
            }
        }

        let response = self
            .client
            .exec_chat(&self.model_name, chat_req, None)
            .await
            .map_err(|e| CommonError::CommonError(format!("LLM request failed: {e}")))?;

        response
            .first_text()
            .map(ToString::to_string)
            .ok_or_else(|| CommonError::CommonError("LLM response has no text content".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::LLMClient;
    use common_base::error::common::CommonError;
    use common_config::config::{LLMClientConfig, LLMPlatform};

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY and real network access"]
    async fn test_openai_chat() -> Result<(), CommonError> {
        let token = std::env::var("OPENAI_API_KEY").map_err(|_| {
            CommonError::CommonError("OPENAI_API_KEY environment variable is required".to_string())
        })?;

        let config = LLMClientConfig {
            platform: LLMPlatform::OpenAI,
            model: "gpt-4o-mini".to_string(),
            token: Some(token),
            base_url: std::env::var("OPENAI_BASE_URL").ok(),
        };

        let client = LLMClient::new(config)?;
        let response = client.chat("Return exactly: ok").await?;

        assert!(!response.trim().is_empty());
        Ok(())
    }
}
