use tonic::{Request, Status};

pub fn grpc_intercept(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting request metadata: {:?}", req.metadata());
    println!("Intercepting request remote_addr: {:?}", req.remote_addr());
    println!("Intercepting request extensions: {:?}", req.extensions());

    Ok(req)
}
