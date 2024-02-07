use protocol::storage::storage::{
    shard_service_server::ShardService, CreateSegmentRequest, CreateSegmentResponse,
    CreateShardRequest, CreateShardResponse, ReadRequest, ReadResponse, ShardDetailRequest,
    ShardDetailResponse, WriteRequest, WriteResponse,
};
use tonic::{Request, Response, Status};

pub struct StorageService {}

#[tonic::async_trait]
impl ShardService for StorageService {
    async fn write(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<WriteResponse>, Status> {
        return Ok(Response::new(WriteResponse::default()));
    }

    async fn read(&self, request: Request<ReadRequest>) -> Result<Response<ReadResponse>, Status> {
        return Ok(Response::new(ReadResponse::default()));
    }

    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CreateShardResponse>, Status> {
        return Ok(Response::new(CreateShardResponse::default()));
    }

    async fn create_segment(
        &self,
        request: Request<CreateSegmentRequest>,
    ) -> Result<Response<CreateSegmentResponse>, Status> {
        return Ok(Response::new(CreateSegmentResponse::default()));
    }

    async fn shard_detail(
        &self,
        request: Request<ShardDetailRequest>,
    ) -> Result<Response<ShardDetailResponse>, Status> {
        return Ok(Response::new(ShardDetailResponse::default()));
    }
}
