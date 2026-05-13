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

use crate::client::LLMResult;
use common_base::error::common::CommonError;
use fastembed::{InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::OnceCell;

static FASTEMBED_MODEL: OnceCell<Arc<TextEmbedding>> = OnceCell::const_new();

fn err(msg: impl Into<String>) -> Box<CommonError> {
    Box::new(CommonError::CommonError(msg.into()))
}

/// Initialize the global fastembed model from a local directory.
/// Must be called once at startup before any embed calls.
/// The directory must contain: model.onnx, tokenizer.json, config.json,
/// special_tokens_map.json, tokenizer_config.json
pub fn init(model_dir: &str) -> LLMResult<()> {
    let dir = PathBuf::from(model_dir);

    if !dir.exists() {
        tracing::info!(
            "fastembed model directory '{}' does not exist, skipping initialization",
            model_dir
        );
        return Ok(());
    }

    let read = |filename: &str| -> LLMResult<Vec<u8>> {
        let path = dir.join(filename);
        std::fs::read(&path).map_err(|e| err(format!("failed to read {}: {}", path.display(), e)))
    };

    let tokenizer_files = TokenizerFiles {
        tokenizer_file: read("tokenizer.json")?,
        config_file: read("config.json")?,
        special_tokens_map_file: read("special_tokens_map.json")?,
        tokenizer_config_file: read("tokenizer_config.json")?,
    };

    let model = TextEmbedding::try_new_from_user_defined(
        UserDefinedEmbeddingModel::new(read("model.onnx")?, tokenizer_files),
        InitOptionsUserDefined::default(),
    )
    .map_err(|e| {
        err(format!(
            "failed to init fastembed model from {model_dir}: {e}"
        ))
    })?;

    FASTEMBED_MODEL
        .set(Arc::new(model))
        .map_err(|_| err("fastembed model already initialized"))
}

fn get() -> LLMResult<Arc<TextEmbedding>> {
    FASTEMBED_MODEL.get().cloned().ok_or_else(|| {
        err("fastembed model not initialized, call embedding::fastembed::init() first")
    })
}

pub async fn embed(text: &str) -> LLMResult<Vec<f32>> {
    let model = get()?;
    let input = vec![text.to_string()];
    tokio::task::spawn_blocking(move || {
        model
            .embed(input, None)
            .map_err(|e| err(format!("fastembed embed failed: {e}")))
            .map(|mut v| v.remove(0))
    })
    .await
    .map_err(|e| err(format!("spawn_blocking failed: {e}")))?
}

pub async fn embed_batch(texts: Vec<String>) -> LLMResult<Vec<Vec<f32>>> {
    let model = get()?;
    tokio::task::spawn_blocking(move || {
        model
            .embed(texts, None)
            .map_err(|e| err(format!("fastembed embed_batch failed: {e}")))
    })
    .await
    .map_err(|e| err(format!("spawn_blocking failed: {e}")))?
}
