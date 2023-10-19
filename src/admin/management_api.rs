
pub async fn api_overview_get_handler() -> &'static str {
    "API overview"
}

pub async fn api_cluster_get_handler() -> &'static str {
    "cluster name: cluster_production"
}

pub async fn api_nodes_handler() -> &'static str {
    "cluster nodes: node 1, node 2, node 3"
}

pub async fn api_node_name_handler() -> &'static str {
    "cluster name: node 1"
}
