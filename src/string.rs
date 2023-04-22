use crate::vec::Vec;

#[repr(C)]
pub struct String {
    vec: Vec<u8>,
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        String {
            vec: Vec::from(value.into_bytes()),
        }
    }
}
