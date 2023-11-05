use thiserror::Error;

#[derive(Error,Debug)]
pub enum MetaError {
    #[error("The index of term is smaller than the smallest index")]
    MetaIndexOutRange,
}
