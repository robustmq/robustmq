/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
    use crate::errors::MetaError;


    #[test]
    fn thiserror_to_string(){
        println!("{}",MetaError::LeaderExistsNotAllowElection.to_string());
        println!("{}",MetaError::NodeBeingVotedOn { node_id: 18 }.to_string());
    }
}