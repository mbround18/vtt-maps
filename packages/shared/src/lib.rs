pub mod types;
pub mod utils;

pub fn decode(encoded: String) -> Vec<u8> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(encoded).unwrap()
}
