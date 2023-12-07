#[cfg(test)]
mod tests {
    use broker::network::network::Network;
    use bytes::{Bytes, BytesMut};
    use common::runtime::create_runtime;
    use protocol::{
        mqttv4::{self, MqttV4},
        protocol::{Connect, LastWill, Login},
    };
    use std::{thread::sleep, sync::Arc};
    use std::time::Duration;
    use tokio::{
        io::AsyncWriteExt,
        net::TcpStream, sync::RwLock,
    };

    #[test]
    fn mqtt4_broker() {}

    #[test]
    fn client() {
        let rt = create_runtime("text", 10);
        let guard = rt.enter();
        tokio::spawn(async move {
            let stream = TcpStream::connect("127.0.0.1:9989").await.unwrap();
            let socket = Arc::new(RwLock::new(Box::new(stream)));

            // send connect package
            let write_buf = build_pg_connect();
            let _ = socket.write().await.write_all(&write_buf).await;


            // read connack recv
            let prot = MqttV4::new();
            let mut network = Network::new(socket, 2000, 2000, 2000, 3000, prot);            println!("{}",2);
            loop{
                match network.read().await {
                    Ok(pkg) => {
                        println!("receive pkg: {:?}", pkg);
                    }
                    Err(e) => {
                        println!("receive pkg err: {:?}", e);
                    }
                }
            }
        });

        drop(guard);
        sleep(Duration::from_secs(10));
    }

    /// Build the connect content package for the mqtt4 protocol
    fn build_pg_connect() -> BytesMut {
        let client_id = String::from("test_client_id");
        let mut buff_write: BytesMut = BytesMut::new();
        let login = Some(Login {
            username: "lobo".to_string(),
            password: "123456".to_string(),
        });
        let lastwill = Some(LastWill {
            topic: Bytes::from("topic1"),
            message: Bytes::from("connection content"),
            qos: protocol::protocol::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id: client_id,
            clean_session: true,
        };
        let _ = mqttv4::connect::write(&connect, &login, &lastwill, &mut buff_write);
        return buff_write;
    }
}
