use anyhow::{Result, anyhow};
use std::env;
use tracing::{info, warn};

/// Initializes and validates admin configuration on startup.
///
/// This function:
/// 1. Checks if `DISCORD_ADMIN_USER_ID` is set (legacy single admin ID)
/// 2. If set, uses it if `DISCORD_ADMIN_IDS` is not already set
/// 3. Validates that at least one admin is configured
/// 4. Returns the effective admin IDs for use in the application
pub fn initialize_admin_config() -> Result<String> {
    let legacy_admin_id = env::var("DISCORD_ADMIN_USER_ID").ok();
    let admin_ids_env = env::var("DISCORD_ADMIN_IDS").ok();

    let effective_admin_ids = match (legacy_admin_id, admin_ids_env) {
        // Case 1: Both legacy and new are set - use new but warn about redundancy
        (Some(legacy), Some(new)) => {
            if !new.contains(&legacy) {
                warn!(
                    "DISCORD_ADMIN_USER_ID ({}) differs from DISCORD_ADMIN_IDS ({}). Using DISCORD_ADMIN_IDS.",
                    legacy, new
                );
            } else {
                info!("DISCORD_ADMIN_USER_ID is now configured via DISCORD_ADMIN_IDS");
            }
            new
        }
        // Case 2: Only legacy is set - use it
        (Some(legacy), None) => {
            info!(
                "Using DISCORD_ADMIN_USER_ID (legacy format) as admin: {}",
                legacy
            );
            legacy
        }
        // Case 3: Neither set - error out
        (None, None) => {
            return Err(anyhow!(
                "No admin user configured. Set either DISCORD_ADMIN_USER_ID or DISCORD_ADMIN_IDS environment variable."
            ));
        }
        // Case 4: Only new is set - all good
        (None, Some(ids)) => {
            info!("Admin users configured via DISCORD_ADMIN_IDS: {}", ids);
            ids
        }
    };

    // Validate that at least one admin ID is configured and non-empty
    let admin_list: Vec<&str> = effective_admin_ids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if admin_list.is_empty() {
        return Err(anyhow!(
            "No valid admin IDs found in admin configuration. Ensure the format is comma-separated Discord user IDs (e.g., '140642000969007104' or '140642000969007104,987654321098765432')"
        ));
    }

    info!(
        "✓ Admin configuration validated. {} admin user(s) configured: {}",
        admin_list.len(),
        admin_list.join(", ")
    );

    Ok(effective_admin_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_with_legacy_only() {
        unsafe {
            env::set_var("DISCORD_ADMIN_USER_ID", "123456789");
            env::remove_var("DISCORD_ADMIN_IDS");
        }

        let result = initialize_admin_config();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "123456789");
    }

    #[test]
    fn test_initialize_with_new_format() {
        unsafe {
            env::remove_var("DISCORD_ADMIN_USER_ID");
            env::set_var("DISCORD_ADMIN_IDS", "123456789,987654321");
        }

        let result = initialize_admin_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_with_both_matching() {
        unsafe {
            env::set_var("DISCORD_ADMIN_USER_ID", "123456789");
            env::set_var("DISCORD_ADMIN_IDS", "123456789,987654321");
        }

        let result = initialize_admin_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_with_neither_set() {
        unsafe {
            env::remove_var("DISCORD_ADMIN_USER_ID");
            env::remove_var("DISCORD_ADMIN_IDS");
        }

        let result = initialize_admin_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_with_empty_ids() {
        unsafe {
            env::remove_var("DISCORD_ADMIN_USER_ID");
            env::set_var("DISCORD_ADMIN_IDS", "");
        }

        let result = initialize_admin_config();
        assert!(result.is_err());
    }
}
