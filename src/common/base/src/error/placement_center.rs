use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlacementCenterError {
    #[error("Description The interface {0} submitted logs to the commit log")]
    RaftLogCommitTimeout(String),
}
