mod common;

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use paho_mqtt::{Message, Properties, SubscribeOptionsBuilder, QOS_0, QOS_1};

    use crate::common::{broker_addr, connect_server34};

    #[tokio::test]
    async fn client5_subscribe_test() {
        let num = 10;
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();
        let cli = connect_server34(mqtt_version, &client_id, &addr);

        let topic = format!("/tests/{}", unique_id());
        let opts = SubscribeOptionsBuilder::new().finalize();
        let props = Properties::new();
        cli.subscribe_with_options(topic.clone(), QOS_0, opts, props)
            .unwrap();
        let rx = cli.start_consuming();

        for i in 0..num {
            let msg = Message::new(topic.clone(), format!("mqtt {i} message"), QOS_1);
            match cli.publish(msg) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    assert!(false);
                }
            }
        }
    }
}
