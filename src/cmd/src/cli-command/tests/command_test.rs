use super::*;

use mockall::*;
use mockall::predicate::*;

#[automock]
#[async_trait]
trait MqttBrokerCommandTrait {
    async fn start(&self, params: MqttCliCommandParam);
    async fn status(&self, client_pool: Arc<ClientPool>, params: MqttCliCommandParam);
    async fn create_user(
        &self,
        client_pool: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: CreateUserRequest,
    );
    // Add other methods as needed...
}
