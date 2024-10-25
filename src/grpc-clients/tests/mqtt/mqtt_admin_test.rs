#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use grpc_clients::{mqtt::admin::call::{create_user, delete_user, list_user}, poll::ClientPool};
    use metadata_struct::mqtt::user::MqttUser;
    use protocol::broker_mqtt::broker_mqtt_admin::{CreateUserRequest, DeleteUserRequest, ListUserRequest};

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn user_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let user_name: String = "user1".to_string();
        let password: String = "123456".to_string();

        //create user
        let user = CreateUserRequest {
            username: user_name.clone(),
            password: password.clone(),
            is_superuser: false,
        };

        match create_user(client_poll.clone(), addrs.clone(), user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        //
        match list_user(client_poll.clone(), addrs.clone(), ListUserRequest{}).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = true;
                    }
                }
                assert!(flag, "user1 has been created");
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        match delete_user(client_poll.clone(), addrs.clone(), DeleteUserRequest{ username: user.username.clone() }).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        match list_user(client_poll.clone(), addrs.clone(), ListUserRequest{}).await {
            Ok(data) => {
                let mut flag = true;
                for raw in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(raw.as_slice()).unwrap();
                    if user.username == mqtt_user.username {
                        flag = false;
                    }
                }
                assert!(flag, "user1 should be deleted");
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        };


    }
}