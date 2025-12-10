//! Authentication and authorization middleware helper functions.
//!
//! These helpers are designed to be used with Actix-web extractors
//! and route guards to enforce authentication and role-based access control.

use crate::auth::models::UserRole;

/// Check if a user has admin privileges
#[allow(dead_code)]
pub fn is_admin_user(role: UserRole) -> bool {
    role == UserRole::Admin
}

/// Check if a user has contributor or admin privileges
#[allow(dead_code)]
pub fn is_contributor_or_admin(role: UserRole) -> bool {
    role == UserRole::Contributor || role == UserRole::Admin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin_user_with_admin_role() {
        assert!(is_admin_user(UserRole::Admin));
    }

    #[test]
    fn test_is_admin_user_with_non_admin_role() {
        assert!(!is_admin_user(UserRole::Guest));
        assert!(!is_admin_user(UserRole::Contributor));
    }

    #[test]
    fn test_is_contributor_or_admin() {
        assert!(is_contributor_or_admin(UserRole::Admin));
        assert!(is_contributor_or_admin(UserRole::Contributor));
        assert!(!is_contributor_or_admin(UserRole::Guest));
    }
}
