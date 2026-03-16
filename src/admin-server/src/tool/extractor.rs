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

use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    Json,
};
use common_base::http_response::error_response;
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                error_response(format!("JSON parsing failed: {}", err)),
            )
        })?;

        value.validate().map_err(|err| {
            let error_message = format_validation_errors(&err);
            (
                StatusCode::BAD_REQUEST,
                error_response(format!("Parameter validation failed: {}", error_message)),
            )
        })?;

        Ok(ValidatedJson(value))
    }
}

fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
    let mut messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = if let Some(msg) = &error.message {
                msg.to_string()
            } else {
                format!("Field '{}' validation failed: {:?}", field, error.code)
            };
            messages.push(message);
        }
    }

    if messages.is_empty() {
        "Unknown validation error".to_string()
    } else {
        messages.join("; ")
    }
}
