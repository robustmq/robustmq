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
use common_config::config::{LLMConfig, LLMPlatform};
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{AuthData, Endpoint};
use genai::{Client, ModelIden, ServiceTarget};

pub type LLMResult<T> = Result<T, Box<CommonError>>;

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
    pub fn new(config: LLMConfig) -> LLMResult<Self> {
        config
            .validate()
            .map_err(|e| Box::new(CommonError::CommonError(e)))?;

        let platform = config.platform.as_ref().ok_or_else(|| {
            Box::new(CommonError::CommonError(
                "platform is not configured".to_string(),
            ))
        })?;
        let adapter_kind = platform_to_adapter_kind(platform);
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

        let model_name = config.model.clone().unwrap_or_default();
        Ok(Self { model_name, client })
    }

    pub async fn chat(&self, prompt: &str) -> LLMResult<String> {
        self.chat_with_system(None, prompt).await
    }

    pub async fn embed(&self, input: &str) -> LLMResult<Vec<f32>> {
        if input.trim().is_empty() {
            return Err(Box::new(CommonError::CommonError(
                "embed input cannot be empty".to_string(),
            )));
        }

        let response = self
            .client
            .embed(&self.model_name, input, None)
            .await
            .map_err(|e| Box::new(CommonError::CommonError(format!("LLM embed failed: {e}"))))?;

        response.first_vector().cloned().ok_or_else(|| {
            Box::new(CommonError::CommonError(
                "embed response has no vector".to_string(),
            ))
        })
    }

    pub async fn embed_batch(&self, inputs: Vec<String>) -> LLMResult<Vec<Vec<f32>>> {
        if inputs.is_empty() {
            return Err(Box::new(CommonError::CommonError(
                "embed_batch inputs cannot be empty".to_string(),
            )));
        }

        let response = self
            .client
            .embed_batch(&self.model_name, inputs, None)
            .await
            .map_err(|e| {
                Box::new(CommonError::CommonError(format!(
                    "LLM embed_batch failed: {e}"
                )))
            })?;

        Ok(response.into_vectors())
    }

    pub async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        prompt: &str,
    ) -> LLMResult<String> {
        if prompt.trim().is_empty() {
            return Err(Box::new(CommonError::CommonError(
                "prompt cannot be empty".to_string(),
            )));
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
            .map_err(|e| Box::new(CommonError::CommonError(format!("LLM request failed: {e}"))))?;

        response
            .first_text()
            .map(ToString::to_string)
            .ok_or_else(|| {
                Box::new(CommonError::CommonError(
                    "LLM response has no text content".to_string(),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::LLMClient;
    use common_config::config::{LLMConfig, LLMPlatform};

    #[tokio::test]
    #[ignore = "requires OPENAI_API_KEY and real network access"]
    async fn test_openai_chat() -> super::LLMResult<()> {
        let token = std::env::var("OPENAI_API_KEY").map_err(|_| {
            Box::new(common_base::error::common::CommonError::CommonError(
                "OPENAI_API_KEY environment variable is required".to_string(),
            ))
        })?;

        let config = LLMConfig {
            embedding: Some("api".to_string()),
            embedding_model_path: None,
            platform: Some(LLMPlatform::OpenAI),
            model: Some("gpt-4o-mini".to_string()),
            token: Some(token),
            base_url: std::env::var("OPENAI_BASE_URL").ok(),
        };

        let client = LLMClient::new(config)?;
        let response = client.chat("Return exactly: ok").await?;

        assert!(!response.trim().is_empty());
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires local ollama service and model pre-pulled"]
    async fn test_ollama_chat() -> super::LLMResult<()> {
        let config = LLMConfig {
            embedding: Some("fastembed".to_string()),
            embedding_model_path: None,
            platform: Some(LLMPlatform::Ollama),
            model: Some(std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:3b".to_string())),
            token: None,
            base_url: std::env::var("OLLAMA_BASE_URL").ok(),
        };

        let client = LLMClient::new(config)?;
        let response = client
            .chat("Reply with a short phrase that includes the word robustmq.")
            .await?;

        assert!(!response.trim().is_empty());
        Ok(())
    }
}
