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

use std::{future::Future, time::Duration};

use tokio::{select, sync::broadcast};

use crate::core::error::JournalServerError;

pub async fn loop_select<F, Fut>(ac_fn: F, tick_secs: u64, stop_sx: &broadcast::Sender<bool>)
where
    F: FnOnce() -> Fut + Copy,
    Fut: Future<Output = Result<(), JournalServerError>>,
{
    let mut stop_recv = stop_sx.subscribe();
    let mut internal = tokio::time::interval(Duration::from_secs(tick_secs));
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        break;
                    }
                }
            }
            _ = internal.tick() => {
                let _ = ac_fn().await;
            }
        }
    }
}
