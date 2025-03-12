use tonic::{Request, Status};

pub fn grpc_intercept(req: Request<()>) -> Result<Request<()>, Status> {

    Ok(req)
}
