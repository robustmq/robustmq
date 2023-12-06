use protocol::robust::broker::{
    broker_service_server::BrokerService, StopBrokerReply, StopBrokerRequest,
};
use tonic::{Request, Response, Status};

pub struct GrpcBrokerServices {}

impl GrpcBrokerServices {
    pub fn new() -> Self {
        return GrpcBrokerServices {};
    }
}

#[tonic::async_trait]

impl BrokerService for GrpcBrokerServices {
    /// stop broker
    async fn stop_broker(
        &self,
        _: Request<StopBrokerRequest>,
    ) -> Result<Response<StopBrokerReply>, Status> {
        let mut reply = StopBrokerReply::default();

        return Ok(Response::new(reply));
    }
}
