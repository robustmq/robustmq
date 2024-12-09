pub fn bool_to_vec(b: bool) -> Vec<u8> {
    if b {
        vec![0x01]
    } else {
        vec![0x00]
    }
}

pub fn vec_to_bool(v: &Vec<u8>) -> bool {
    matches!(v.as_slice(), [0x01])
}
