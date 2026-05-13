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

use std::sync::Arc;

use axum::{extract::State, http::header, response::Response};
use bytes::Bytes;

use crate::state::HttpState;

pub async fn pprof_flamegraph(State(state): State<Arc<HttpState>>) -> Response {
    let Some(guard) = &state.pprof_guard else {
        return Response::builder()
            .status(200)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(axum::body::Body::from(Bytes::from(
                "pprof is disabled. Set runtime.pprof_enable = true in config to enable it.",
            )))
            .unwrap();
    };

    if let Ok(report) = guard.report().build() {
        let mut buf = Vec::new();
        if report.flamegraph(&mut buf).is_ok() {
            return Response::builder()
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .body(axum::body::Body::from(buf))
                .unwrap();
        }
    }

    Response::builder()
        .status(500)
        .body(axum::body::Body::from(Bytes::from(
            "Failed to generate flamegraph",
        )))
        .unwrap()
}
