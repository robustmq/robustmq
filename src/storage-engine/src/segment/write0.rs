use std::sync::Arc;
use std::time::Duration;

use crate::core::error::StorageEngineError;
use crate::segment::write::SegmentWriteResp;
use dashmap::DashMap;
use protocol::storage::storage_engine_record::StorageEngineRecord;
use std::hash::Hasher;
use tokio::select;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};
use tracing::error;
use twox_hash::XxHash32;

#[derive(Clone)]
pub struct SegmentWrite {
    pub data_sender: Sender<SegmentWriteData>,
    pub stop_sender: broadcast::Sender<bool>,
}

/// the data to be sent to the segment write thread
pub struct SegmentWriteData {
    pub data: Vec<StorageEngineRecord>,
    pub resp_sx: oneshot::Sender<SegmentWriteResp>,
}

pub struct WriteManager {
    io_num: u32,
    io_thread: DashMap<u32, Sender<SegmentWriteData>>,
    stop_send: broadcast::Sender<bool>,
}

impl WriteManager {
    pub fn new(io_num: u32, stop_send: broadcast::Sender<bool>) -> Self {
        WriteManager {
            io_num,
            stop_send,
            io_thread: DashMap::with_capacity(2),
        }
    }

    pub fn start(&self) {
        for i in 0..self.io_num {
            let (data_sender, data_recv) = mpsc::channel::<SegmentWriteData>(1000);
            let io_work = Arc::new(IoWork::new(i));
            create_io_thread(io_work, data_recv, self.stop_send.clone());
            self.io_thread.insert(i, data_sender);
        }
    }

    pub async fn write(
        &self,
        shard: &str,
        data_list: &Vec<StorageEngineRecord>,
    ) -> Result<SegmentWriteResp, StorageEngineError> {
        if self.io_thread.len() == 0 {
            return Err(StorageEngineError::NoAvailableIoThread);
        }

        let work_num = self.hash_string(shard);
        let Some(sender) = self.io_thread.get(&work_num) else {
            return Err(StorageEngineError::NoAvailableIoThread);
        };

        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();
        let data = SegmentWriteData {
            data: data_list.clone(),
            resp_sx: sx,
        };
        sender.send(data).await?;

        let time_res: Result<SegmentWriteResp, oneshot::error::RecvError> =
            timeout(Duration::from_secs(30), rx).await?;
        Ok(time_res?)
    }

    fn hash_string(&self, shard: &str) -> u32 {
        let mut hasher = XxHash32::default();
        hasher.write(shard.as_bytes());
        hasher.finish() as u32
    }
}

pub struct IoWork {
    io_seq: u32,
    offset_data: DashMap<String, u64>,
}

impl IoWork {
    pub fn new(io_seq: u32) -> Self {
        IoWork {
            io_seq,
            offset_data: DashMap::with_capacity(16),
        }
    }

    pub fn next_offset(&self) {}
}

pub fn create_io_thread(
    io_work: Arc<IoWork>,
    mut data_recv: Receiver<SegmentWriteData>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut stop_recv = stop_send.subscribe();

        // batch recv data
        let mut results = Vec::new();
        loop {
            match data_recv.try_recv() {
                Ok(data) => {
                    results.push(data);
                    if results.len() == 10 {
                        break;
                    }
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_millis(10)).await;
                    break;
                }
                Err(e) => {
                    error!("{}", e.to_string());
                }
            }
        }

        // 
    });
}
