use common_base::error::common::CommonError;
use metadata::AclMetadata;

pub mod metadata;

pub fn check_resource_acl(acl_metadata: &AclMetadata) -> Result<bool, CommonError> {
    // check super user
    if check_super_user() {
        return Ok(true);
    }

    // check blacklist
    if check_black_list() {
        return Ok(true);
    }

    // check acl
    if check_acl() {
        return Ok(true);
    }
    return Ok(true);
}

fn check_super_user() -> bool {
    return true;
}

fn check_black_list() -> bool {
    return true;
}

fn check_acl() -> bool {
    return true;
}
