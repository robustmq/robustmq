use protocol::storage::storage::{
    storage_engine_service_server::StorageEngineService, CreateShardRequest, CreateShardResponse,
    ReadRequest, ReadResponse, ShardDetailRequest, ShardDetailResponse, WriteRequest,
    WriteResponse,
};
use tonic::{Request, Response, Status};

pub struct StorageService {}

impl StorageService{
    pub fn new() -> Self{
        return StorageService{};
    }
}

#[tonic::async_trait]
impl StorageEngineService for StorageService {
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

    async fn describe_shard(
        &self,
        request: Request<ShardDetailRequest>,
    ) -> Result<Response<ShardDetailResponse>, Status> {
        return Ok(Response::new(ShardDetailResponse::default()));
    }
}
