use thiserror::Error;

#[derive(Error,Debug)]
pub enum MetaError {
    #[error("Operation cannot be initiated because the Leader exists in the cluster")]
    LeaderExistsNotAllowElection,
    
    #[error("Node is currently in the voting state. The target node ID is : {node_id:?}")]
    NodeBeingVotedOn{
        node_id: i32
    },

    #[error("Node ID is unavailable. The data format may be incorrect. The node id is : {node_id:?}")]
    UnavailableNodeId{
        node_id: i32
    },
}


#[cfg(test)]
mod tests{
    use crate::meta::errors::MetaError;

    #[test]
    fn thiserror_to_string(){
        println!("{}",MetaError::LeaderExistsNotAllowElection.to_string());
        println!("{}",MetaError::NodeBeingVotedOn { node_id: 18 }.to_string());
    }
}