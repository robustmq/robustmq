use std::sync::atomic::AtomicUsize;

static RANGE_AT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn meta_node_range_addr(addrs: Vec<String>) -> String {
    let idx: usize =
    RANGE_AT_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)% addrs.len();
    let addr = addrs.get(idx);
    return addr.unwrap().into();
}

#[cfg(test)]
mod tests {
    use crate::tools::meta_node_range_addr;

    #[test]
    fn range_addr_test() {
        let mut v: Vec<String> = Vec::new();
        v.push("X".to_string());
        v.push("Y".to_string());
        v.push("Z".to_string());

        assert_eq!(meta_node_range_addr(v.clone()), "X".to_string());
        assert_eq!(meta_node_range_addr(v.clone()), "Y".to_string());
        assert_eq!(meta_node_range_addr(v.clone()), "Z".to_string());
        assert_eq!(meta_node_range_addr(v.clone()), "X".to_string());
    }
}
