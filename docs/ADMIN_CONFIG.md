# Admin Configuration Auto-Assignment

This system automatically configures and validates Discord admin users at application startup.

## How It Works

### 1. **Automatic Configuration on Startup**

When the application starts, `initialize_admin_config()` is called in `main.rs` before any other critical initialization. It:

- Checks for `DISCORD_ADMIN_USER_ID` (legacy single ID format)
- Checks for `DISCORD_ADMIN_IDS` (new comma-separated format)
- Returns the effective admin IDs being used

### 2. **Configuration Resolution**

The function handles multiple scenarios:

| Scenario                                     | Behavior                                     |
| -------------------------------------------- | -------------------------------------------- |
| Only `DISCORD_ADMIN_USER_ID` set             | Uses it as the admin ID                      |
| Only `DISCORD_ADMIN_IDS` set                 | Uses it as-is                                |
| Both set, legacy ID in DISCORD_ADMIN_IDS     | Uses DISCORD_ADMIN_IDS (both are compatible) |
| Both set, legacy ID NOT in DISCORD_ADMIN_IDS | Warns and uses DISCORD_ADMIN_IDS             |
| Neither set                                  | **Fails startup with clear error message**   |
| Either is empty after trimming               | **Fails startup with validation error**      |

### 3. **User Assignment During OAuth**

When a user logs in via Discord OAuth (`discord_callback` in `auth/routes.rs`):

1. The application checks `DISCORD_ADMIN_IDS` (which is validated at startup)
2. User role is resolved using `resolve_insert_role()` which calls `id_in_env("DISCORD_ADMIN_IDS", discord_id)`
3. User is assigned as Admin, Contributor, or Guest accordingly

## Configuration

### Using the New Format (Recommended)

Set `DISCORD_ADMIN_IDS` as a comma-separated list:

```bash
export DISCORD_ADMIN_IDS="140642000969007104,987654321098765432"
```

### Using the Legacy Format (Still Supported)

Set `DISCORD_ADMIN_USER_ID`:

```bash
export DISCORD_ADMIN_USER_ID="140642000969007104"
```

### Environment File (.env)

Add to `.env`:

```dotenv
DISCORD_ADMIN_USER_ID="140642000969007104"
# OR
DISCORD_ADMIN_IDS="140642000969007104,987654321098765432"
```

## Validation

The system performs the following checks at startup:

- ✅ At least one admin ID is configured
- ✅ Admin IDs are non-empty after trimming whitespace
- ✅ Admin ID format is valid (Discord user IDs are numeric strings)
- ✅ Logs all configured admins for verification

### Example Startup Output

```text
✓ Admin configuration validated. 1 admin user(s) configured: 140642000969007104
```

Or with multiple admins:

```text
✓ Admin configuration validated. 2 admin user(s) configured: 140642000969007104, 987654321098765432
```

## Error Handling

If configuration is invalid, the application exits with a clear error:

```text
❌ Admin configuration initialization failed: No admin user configured.
Set either DISCORD_ADMIN_USER_ID or DISCORD_ADMIN_IDS environment variable.
```

## Integration Points

1. **Startup** (`src/main.rs`): Validates config before other initialization
2. **OAuth Flow** (`src/auth/routes.rs`): Uses validated config during user login
3. **User Roles** (`src/auth/models.rs`): User is assigned Admin role if their Discord ID matches

## Testing

Run tests with:

```bash
cargo test utils::admin_init
```

Test cases:

- ✅ Legacy format only
- ✅ New format only
- ✅ Both formats with matching ID
- ✅ Neither format (should fail)
- ✅ Empty ID values (should fail)

## Migration Path

If you're currently using `DISCORD_ADMIN_USER_ID`:

1. Keep your current setup - it continues to work
2. Optionally migrate to `DISCORD_ADMIN_IDS` for multiple admin support:

```bash
export DISCORD_ADMIN_IDS="your-admin-id-1,your-admin-id-2"
```

Both formats can coexist; the new format takes precedence if both are set with compatible values.
