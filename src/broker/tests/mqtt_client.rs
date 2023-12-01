#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use common_base::runtime::create_runtime;
    use protocol::{mqttv4, Connect, LastWill, Login};
    use std::thread::sleep;
    use std::time::Duration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[test]
    fn mqtt4_broker() {}

    #[test]
    fn client() {
        let rt = create_runtime("text", 10);
        let guard = rt.enter();
        tokio::spawn(async move {
            let mut stream = TcpStream::connect("127.0.0.1:8768").await.unwrap();
            
            // send connect package
            let write_buf = build_pg_connect();
            let _ = stream.write_all(&write_buf).await;

            // read recv
            let mut read_buf = BytesMut::with_capacity(20);
            match stream.read_buf(&mut read_buf).await {
                Ok(_) => {
                    let content = String::from_utf8_lossy(&read_buf).to_string();
                    println!("receive:{}", content)
                }
                Err(err) => {
                    println!("err:{:?}", err)
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
            qos: protocol::QoS::AtLeastOnce,
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
