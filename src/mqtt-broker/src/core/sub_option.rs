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

use dashmap::DashMap;
use protocol::mqtt::common::RetainHandling;

// No Local
// The only values available for No Local are 0 and 1, where 1 means that the server cannot forward the message to the client that posted it, and 0 is the opposite.
pub fn is_send_msg_by_bo_local(no_local: bool, client_id: &str, msg_client_id: &str) -> bool {
    if no_local && client_id == msg_client_id {
        return false;
    }
    true
}

// Retain As Published
// Again, the only values that can be assigned to Retain As Published are 0 and 1, where 1 indicates that the server needs to keep the Retain flag when forwarding application messages to the subscription, and 0 indicates that it must be removed.
pub fn get_retain_flag_by_retain_as_published(
    retain_as_published: bool,
    msg_retain_flag: bool,
) -> bool {
    if retain_as_published {
        return msg_retain_flag;
    }
    false
}

// Retain Handling
pub fn is_send_retain_msg_by_retain_handling(
    path: &str,
    retain_handling: &RetainHandling,
    is_new_subs: &DashMap<String, bool>,
) -> bool {
    println!("is_new_subs:{:?}", is_new_subs);
    println!("retain_handling:{:?}", retain_handling);

    if *retain_handling == RetainHandling::OnEverySubscribe {
        return true;
    }

    if *retain_handling == RetainHandling::Never {
        return false;
    }

    if *retain_handling == RetainHandling::OnNewSubscribe {
        let is_new_sub = if let Some(new_sub) = is_new_subs.get(path) {
            new_sub.to_owned()
        } else {
            true
        };

        if is_new_sub {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use dashmap::DashMap;
    use protocol::mqtt::common::RetainHandling;

    use crate::core::sub_option::{
        get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local,
        is_send_retain_msg_by_retain_handling,
    };

    #[tokio::test]
    async fn is_send_msg_by_bo_local_test() {
        let no_local = true;
        let client_id = "client_id";
        let msg_client_id = "client_id";
        assert!(!is_send_msg_by_bo_local(no_local, client_id, msg_client_id));

        let no_local = true;
        let client_id = "client_id";
        let msg_client_id = "client_id_1";
        assert!(is_send_msg_by_bo_local(no_local, client_id, msg_client_id));

        let no_local = false;
        let client_id = "client_id";
        let msg_client_id = "client_id";
        assert!(is_send_msg_by_bo_local(no_local, client_id, msg_client_id));

        let no_local = false;
        let client_id = "client_id";
        let msg_client_id = "client_id_1";
        assert!(is_send_msg_by_bo_local(no_local, client_id, msg_client_id));
    }

    #[tokio::test]
    async fn get_retain_flag_by_retain_as_published_test() {
        let retain_as_published = true;
        let msg_retain_flag = true;
        assert!(get_retain_flag_by_retain_as_published(
            retain_as_published,
            msg_retain_flag
        ));

        let retain_as_published = true;
        let msg_retain_flag = false;
        assert!(!get_retain_flag_by_retain_as_published(
            retain_as_published,
            msg_retain_flag
        ));

        let retain_as_published = false;
        let msg_retain_flag = true;
        assert!(!get_retain_flag_by_retain_as_published(
            retain_as_published,
            msg_retain_flag
        ));

        let retain_as_published = false;
        let msg_retain_flag = false;
        assert!(!get_retain_flag_by_retain_as_published(
            retain_as_published,
            msg_retain_flag
        ));
    }

    #[tokio::test]
    async fn is_send_retain_msg_by_retain_handling_test() {
        let path = "path";
        let retain_handling = RetainHandling::OnEverySubscribe;
        let is_new_subs = DashMap::new();
        assert!(is_send_retain_msg_by_retain_handling(
            path,
            &retain_handling,
            &is_new_subs
        ));

        let path = "path";
        let retain_handling = RetainHandling::Never;
        let is_new_subs = DashMap::new();
        assert!(!is_send_retain_msg_by_retain_handling(
            path,
            &retain_handling,
            &is_new_subs
        ));

        let path = "path";
        let retain_handling = RetainHandling::OnNewSubscribe;
        let is_new_subs = DashMap::new();
        is_new_subs.insert(path.to_string(), true);
        assert!(is_send_retain_msg_by_retain_handling(
            path,
            &retain_handling,
            &is_new_subs
        ));

        let path = "path";
        let retain_handling = RetainHandling::OnNewSubscribe;
        let is_new_subs = DashMap::new();
        is_new_subs.insert(path.to_string(), false);
        assert!(!is_send_retain_msg_by_retain_handling(
            path,
            &retain_handling,
            &is_new_subs
        ));
    }
}
