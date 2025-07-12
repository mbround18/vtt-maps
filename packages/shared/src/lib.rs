pub mod types;
pub mod utils;

/// Decodes a base64 encoded string into bytes.
///
/// # Panics
/// Panics if the base64 string is invalid or cannot be decoded.
#[must_use]
pub fn decode(encoded: String) -> Vec<u8> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(encoded).unwrap()
}
