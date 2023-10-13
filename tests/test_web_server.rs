use std::net::TcpListener;

#[test]
fn test_web_server() {
    //listen to the address specified: 127.0.0.1:7878'
    println!("Server started  !");
    
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Server listened  !");

    for stream in listener.incoming(){
        let stream = stream.unwrap();
        println!("Connection established !");
    }
}
