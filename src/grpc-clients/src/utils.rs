use std::{future::Future, time::Duration};

use common_base::error::common::CommonError;
use log::error;
use tokio::time::sleep;

use crate::{pool::ClientPool, retry_sleep_time, retry_times};

pub(crate) async fn retry_call<'a, F, Fut, Req, Res>(
    client_pool: &'a ClientPool,
    addrs: &'a [String],
    request: Req,
    call_once: F,
) -> Result<Res, CommonError> 
where 
    F: Fn(&'a ClientPool, &'a str, Req) -> Fut + 'static,
    Fut: Future<Output = Result<Res, CommonError>>,
    Req: Clone,
{
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }

    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = &addrs[index];
        let result = call_once(client_pool, addr, request.clone()).await;

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                error!("{}", e);
                if times > retry_times() {
                    return Err(e);
                }
                times += 1;
            }
        }

        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}