use crate::cache::cluster::ClusterCache;
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub fn calc_share_sub_leader(
    cluster_name: String,
    group_name: String,
    cluster_cache: Arc<ClusterCache>,
) -> Result<u64, RobustMQError> {
    if let Some(cluster) = cluster_cache.cluster_list.get(&cluster_name) {
        

        // todo The node where the Share Sub Leader is located is calculated based on the cluster load
        // return Ok(cluster.nodes.first().unwrap().clone());
        return Ok(7);
    }
    return Err(RobustMQError::ClusterDoesNotExist);
}
