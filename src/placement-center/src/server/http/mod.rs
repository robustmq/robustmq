pub mod index;
pub mod mqtt;
pub mod server;

pub(crate) fn v1_path(path: &str) -> String {
    return format!("/v1{}", path);
}

pub(crate) fn path_get(path: &str) -> String {
    return format!("{}/get", path);
}

pub(crate) fn path_create(path: &str) -> String {
    return format!("{}/create", path);
}

pub(crate) fn path_update(path: &str) -> String {
    return format!("{}/update", path);
}

pub(crate) fn path_delete(path: &str) -> String {
    return format!("{}/delete", path);
}

pub(crate) fn path_list(path: &str) -> String {
    return format!("{}/list", path);
}
