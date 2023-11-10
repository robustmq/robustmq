use thiserror::Error;

#[derive(Error,Debug)]
pub enum MetaError {
    #[error("This operation cannot be initiated because the Leader exists in the cluster")]
    NotAllowElection,
}
