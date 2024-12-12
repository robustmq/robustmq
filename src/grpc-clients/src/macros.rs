macro_rules! impl_retriable_request {
    ($req:ty, $client:ty, $res:ty, $getter:ident, $op:ident) => {
        impl $crate::utils::RetriableRequest for $req {
            type Client = $client;
            type Response = $res;
            type Error = common_base::error::common::CommonError;

            async fn get_client(pool: &$crate::pool::ClientPool, addr: std::net::SocketAddr) -> Result<impl std::ops::DerefMut<Target = Self::Client>, Self::Error> {
                pool.$getter(addr).await
            }

            async fn call_once(client: &mut Self::Client, request: Self) -> Result<Self::Response, Self::Error> {
                client.$op(request).await.map(|reply| reply.into_inner())
                    .map_err(Into::into)
            }
        }
    };
}
