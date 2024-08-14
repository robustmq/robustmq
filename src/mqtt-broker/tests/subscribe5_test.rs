mod common;

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, Properties, SubscribeOptionsBuilder, QOS_0, QOS_1};

    use crate::common::{broker_addr, connect_server34};

    #[tokio::test]
    async fn client5_subscribe_test() {
        
    }
}
