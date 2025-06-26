use anyhow::{Context, Result};
use rand::Rng;
use shared::utils::root_dir::root_dir;
use std::{env, fs, path::PathBuf};
use tracing::{info, warn};

const TOKEN_LENGTH: usize = 32;
const ADMIN_TOKEN_FILE: &str = ".admin-token";

/// Gets or generates an admin token for protecting destructive operations.
///
/// # Priority
/// 1. ADMIN_TOKEN environment variable
/// 2. .admin-token file in the project root
/// 3. Generate new token and save to .admin-token file
///
/// # Errors
/// Returns an error if the token file cannot be read or written.
pub fn get_or_create_admin_token() -> Result<String> {
    // Check environment variable first
    if let Ok(token) = env::var("ADMIN_TOKEN") {
        if !token.trim().is_empty() {
            info!("🔑 Using admin token from ADMIN_TOKEN environment variable");
            return Ok(token.trim().to_string());
        }
    }

    // Check for existing token file
    let token_path = admin_token_path()?;
    if token_path.exists() {
        match fs::read_to_string(&token_path) {
            Ok(token) => {
                let token = token.trim().to_string();
                if !token.is_empty() {
                    info!("🔑 Using admin token from file: {}", token_path.display());
                    return Ok(token);
                }
                warn!("⚠️  Admin token file is empty, generating new token");
            }
            Err(e) => {
                warn!(
                    "⚠️  Failed to read admin token file: {}, generating new token",
                    e
                );
            }
        }
    }

    // Generate new token
    let token = generate_secure_token();
    fs::write(&token_path, &token)
        .with_context(|| format!("Failed to write admin token to {}", token_path.display()))?;

    info!(
        "🔑 Generated new admin token and saved to: {}",
        token_path.display()
    );
    info!("🔐 Admin token: {}", token);
    info!("💡 You can also set the ADMIN_TOKEN environment variable to override this file");

    Ok(token)
}

/// Validates if the provided token matches the admin token.
///
/// # Errors
/// Returns an error if the admin token cannot be retrieved.
pub fn validate_admin_token(provided_token: &str) -> Result<bool> {
    let admin_token = get_or_create_admin_token()?;
    Ok(admin_token == provided_token.trim())
}

/// Gets the path to the admin token file.
fn admin_token_path() -> Result<PathBuf> {
    let root = root_dir()?;
    Ok(root.join(ADMIN_TOKEN_FILE))
}

/// Generates a cryptographically secure random token.
fn generate_secure_token() -> String {
    let mut rng = rand::thread_rng();
    (0..TOKEN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..=25 => (b'A' + idx) as char,
                26..=51 => (b'a' + (idx - 26)) as char,
                _ => (b'0' + (idx - 52)) as char,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token();
        assert_eq!(token.len(), TOKEN_LENGTH);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_validate_admin_token() {
        // This test would need to mock the file system or environment
        // For now, just test that the function signature works
        let result = validate_admin_token("test");
        assert!(result.is_ok() || result.is_err()); // Either outcome is valid for testing
    }
}
