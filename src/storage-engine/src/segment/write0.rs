use crate::core::error::StorageEngineError;
use crate::segment::SegmentIdentity;
use bytes::Bytes;
use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{broadcast, oneshot};
use tokio::time::{sleep, timeout};
use tracing::error;
use twox_hash::XxHash32;

#[derive(Clone)]
pub struct SegmentWrite {
    pub data_sender: Sender<WriteChannelData>,
    pub stop_sender: broadcast::Sender<bool>,
}

/// the data to be sent to the segment write thread
pub struct WriteChannelData {
    pub segment_iden: SegmentIdentity,
    pub data: Vec<WriteChannelDataRecord>,
    pub resp_sx: oneshot::Sender<SegmentWriteResp>,
}

pub struct WriteChannelDataRecord {
    pub pkid: u64,
    pub key: String,
    pub value: Bytes,
    pub tags: Vec<String>,
}

/// the response of the write request from the segment write thread
#[derive(Default, Debug)]
pub struct SegmentWriteResp {
    pub offsets: HashMap<u64, u64>,
    pub last_offset: u64,
    pub error: Option<StorageEngineError>,
}

pub struct WriteManager {
    io_num: u32,
    io_thread: DashMap<u32, Sender<WriteChannelData>>,
}

impl WriteManager {
    pub fn new(io_num: u32) -> Self {
        WriteManager {
            io_num,
            io_thread: DashMap::with_capacity(2),
        }
    }

    pub fn start(&self, stop_send: broadcast::Sender<bool>) {
        for i in 0..self.io_num {
            let (data_sender, data_recv) = mpsc::channel::<WriteChannelData>(1000);
            let io_work = Arc::new(IoWork::new(i));
            create_io_thread(io_work, data_recv, stop_send.clone());
            self.io_thread.insert(i, data_sender);
        }
    }

    pub async fn write(
        &self,
        segment_iden: &SegmentIdentity,
        data_list: Vec<WriteChannelDataRecord>,
    ) -> Result<SegmentWriteResp, StorageEngineError> {
        if self.io_thread.len() == 0 {
            return Err(StorageEngineError::NoAvailableIoThread);
        }

        let work_num = self.hash_string(&segment_iden.shard_name);
        let Some(sender) = self.io_thread.get(&work_num) else {
            return Err(StorageEngineError::NoAvailableIoThread);
        };

        let (sx, rx) = oneshot::channel::<SegmentWriteResp>();
        let data = WriteChannelData {
            segment_iden: segment_iden.clone(),
            data: data_list,
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
    mut data_recv: Receiver<WriteChannelData>,
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
