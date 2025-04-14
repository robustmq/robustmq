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

use axum::http::header;
use axum::response::Response;
use axum::{routing::get, Router};
use bytes::Bytes;
use pprof::ProfilerGuard;
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info};

pub async fn start_pprof_monitor(port: u16, frequency: i32) -> Arc<ProfilerGuard<'static>> {
    info!("Starting pprof HTTP server...");
    let guard = Arc::new(ProfilerGuard::new(frequency).expect("Failed to start ProfilerGuard"));
    let guard_clone = Arc::clone(&guard);

    let app = Router::new().route(
        "/flamegraph",
        get(move || generate_flamegraph(guard_clone.clone())),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!(
                "Pprof HTTP Server started successfully, listening port: {}",
                port
            );
            listener
        }
        Err(e) => {
            error!("Failed to bind pprof server: {}", e);
            return guard;
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        error!("pprof HTTP server failed: {}", e);
    }
    guard
}

async fn generate_flamegraph(guard: Arc<ProfilerGuard<'static>>) -> Response {
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
