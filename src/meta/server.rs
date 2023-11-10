use super::{
    node::{Node, NodeRaftState},
    proto::meta::{
        meta_service_server::MetaService, FindLeaderReply, FindLeaderRequest, VoteReply,
        VoteRequest,ReplyCode
    }, errors::MetaError,
};
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

pub struct GrpcService {
    node: Arc<RwLock<Node>>,
}

impl GrpcService {
    pub fn new(node: Arc<RwLock<Node>>) -> Self {
        GrpcService { node: node }
    }
}

#[tonic::async_trait]
impl MetaService for GrpcService {

    async fn find_leader(
        &self,
        _: Request<FindLeaderRequest>,
    ) -> Result<Response<FindLeaderReply>, Status> {

        let node = self.node.read().unwrap();
        let mut reply = FindLeaderReply::default();

        // If the Leader exists in the cluster, the current Leader information is displayed
        if node.raft_state == NodeRaftState::Leader{
            reply.set_code(ReplyCode::Ok);
            reply.leader_id = node.leader_id.unwrap();
            reply.leader_ip = node.leader_ip.unwrap();
            return Ok(Response::new(reply));
        }

        reply.set_code(ReplyCode::Error);
        Ok(Response::new(reply))
    }

    async fn vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteReply>, Status> {
        let node = self.node.read().unwrap();
        let reply = VoteReply::default();
        
        if node.raft_state == NodeRaftState::Leader{
            return Err(Status::already_exists(MetaError::NotAllowElection.to_string()));
        }

        let reply = VoteReply::default();
        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use tokio::runtime::Runtime;
    use tonic_build;

    use crate::meta::proto::meta::{meta_service_client::MetaServiceClient, FindLeaderRequest};
    #[test]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/meta/proto") // you can change the generated code's location
            .compile(
                &["src/meta/proto/meta.proto"],
                &["src/meta/proto/"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }

    #[test]
    fn grpc_client() {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(self.config.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();

        runtime.spawn(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(FindLeaderRequest {});
            let response = client.find_leader(request).await.unwrap();

            println!("response={:?}", response);
        });

        loop {
            thread::sleep(Duration::from_secs(300));
        }
    }
}
