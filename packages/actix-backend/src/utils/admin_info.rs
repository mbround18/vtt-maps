use actix_web::{HttpResponse, Result as ActixResult};
use serde_json::json;

/// Get admin token info without revealing the actual token.
/// This is a public endpoint that shows how to obtain admin access.
pub async fn get_admin_token_info() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Admin token required for destructive operations",
        "how_to_get_token": {
            "file": ".admin-token file in project root (auto-generated on first run)",
            "environment": "ADMIN_TOKEN environment variable (overrides file)",
            "logs": "Check server startup logs for auto-generated token"
        },
        "usage_examples": {
            "header_bearer": "Authorization: Bearer <token>",
            "header_direct": "Authorization: <token>", 
            "query_param": "?admin_token=<token>"
        },
        "protected_endpoints": [
            "POST /api/maps/rebuild - Rebuild search index",
            "DELETE /api/maps/rebuild/clear - Clear rebuild lock"
        ],
        "note": "The token is automatically generated and saved to .admin-token file on first server startup if not provided via environment variable."
    })))
}
