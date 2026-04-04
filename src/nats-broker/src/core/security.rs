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

use common_security::{login::password::password_check_by_login, manager::SecurityManager};
use std::sync::Arc;

/// Check user/password credentials.
///
/// Returns `true` if auth is disabled (`auth_required = false`) or the
/// supplied credentials pass the password check.  Returns `false` if
/// `auth_required` is `true` but no credentials were provided or the
/// password check fails.
pub fn login_check(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    auth_required: bool,
    user: Option<&str>,
    pass: Option<&str>,
) -> bool {
    if !auth_required {
        return true;
    }

    match (user, pass) {
        (Some(u), Some(p)) => password_check_by_login(security_manager, tenant, u, p),
        _ => false,
    }
}
