pub mod types;
pub mod utils;

pub fn decode(encoded: String) -> Vec<u8> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.decode(encoded).unwrap()
}
