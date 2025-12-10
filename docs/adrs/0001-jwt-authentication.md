# ADR-0001: JWT-Based Session Authentication

**Status:** Accepted

## Context

The VTT Maps application needs a secure authentication mechanism for protecting sensitive operations such as map rebuilds and admin functions. The system must:

1. Authenticate users via Discord OAuth
2. Maintain secure sessions for authenticated users
3. Prevent token forgery and tampering attacks
4. Protect against CSRF (Cross-Site Request Forgery) and XSS (Cross-Site Scripting) attacks
5. Support role-based access control (Admin, Contributor, Guest)
6. Avoid database lookups on every request for performance
7. Allow stateless session validation across horizontal scaling

Traditional session-based authentication with server-side session storage would require:

- Database queries on every request
- Distributed session management in clustered environments
- Complex session invalidation logic

We needed a solution that is secure, scalable, and stateless.

## Decision

> In the context of securing a web application with role-based access control, facing the need for both security and stateless scalability, we decided for JWT (JSON Web Tokens) with HMAC-SHA256 signing and secure HTTP-only cookies to achieve cryptographic verification, XSS/CSRF protection, and horizontal scalability, accepting the inability to revoke tokens before expiration, because it provides the best balance of security, performance, and operational simplicity for our use case.

## Considered Options

### Option 1: Traditional Session Cookies + Database

Store sessions in the database with server-side session management.

**Pros:**

- Tokens can be revoked immediately
- Full server control over all sessions
- Simple to understand and implement
- Session data easily updated without re-authentication

**Cons:**

- Requires database query on every request
- Complex session replication in distributed systems
- Session store becomes a bottleneck under load
- Difficult to scale horizontally

**Effort/Complexity:** Low

---

### Option 2: JWT with HS256 Signing (Chosen)

Use cryptographically signed JWT tokens stored in secure HTTP-only cookies.

**Pros:**

- Stateless validation - no database queries needed
- Scales horizontally without session replication
- Cryptographic signature prevents tampering
- Fast validation using just the secret key
- Standard approach used across industry
- Token signature cannot be forged without secret
- Includes expiration (exp) for automatic invalidation
- Contains immutable claims (discord_id, username, role)
- Works well with microservices architecture

**Cons:**

- Tokens cannot be revoked before expiration (60 minutes)
- Secret key compromise affects all tokens
- Token size is larger than session IDs
- Client must send token on every request

**Effort/Complexity:** Medium

---

### Option 3: Opaque Tokens with Token Introspection

Generate random opaque tokens and validate them via introspection endpoint.

**Pros:**

- Tokens can be revoked immediately
- Client receives minimal information
- Simple token format

**Cons:**

- Requires synchronous database/cache lookup on every request
- Introduces additional latency
- Centralized validation point is a bottleneck
- More complex infrastructure

**Effort/Complexity:** High

---

## Implementation Details

### Technology Stack

**JWT Library:** `jsonwebtoken` crate (Rust)
**Signing Algorithm:** HS256 (HMAC-SHA256)
**Secret Key:** `JWT_SECRET` environment variable (never hardcoded)
**Token Format:** Standard JWT with 3 parts separated by dots: `header.payload.signature`

### Security Measures

**JWT Claims:**

```json
{
  "sub": "user-uuid",
  "iss": "vtt-maps",
  "iat": 1702123456,
  "exp": 1702127056,
  "discord_id": "140642000969007104",
  "username": "username",
  "avatar_url": "https://...",
  "role": "admin"
}
```

- `sub` (Subject): Unique user identifier (UUID)
- `iss` (Issuer): Always "vtt-maps" (validated on verify)
- `iat` (Issued At): Unix timestamp of token creation
- `exp` (Expiration): Unix timestamp when token expires (60 minutes default)
- Custom claims: Discord user info and role (immutable in JWT)

**Cookie Configuration:**

- `HttpOnly`: True (prevents XSS JavaScript theft)
- `SameSite`: Strict (prevents CSRF attacks)
- `Secure`: Conditional (HTTPS-only in production)
- `Path`: "/" (available to entire application)
- `Max-Age`: 3600 seconds (60 minutes)

**Signature Verification:**

Every incoming request with a JWT must pass:

1. Signature validation (HMAC verification)
2. Issuer validation (must be "vtt-maps")
3. Expiration check (must not be expired)
4. Algorithm verification (must be HS256)

If any check fails, the request returns 401 Unauthorized.

### Role-Based Access Control

Users are assigned roles during OAuth login:

- **Admin**: Can rebuild maps, manage configuration
- **Contributor**: Can perform contributing tasks
- **Guest**: Read-only access

Role is immutable in the JWT - cannot be changed by modifying the token.

### Middleware Stack (in order)

1. **TracingLogger**: Logs all requests
2. **IdentityMiddleware**: Manages user identity
3. **SessionMiddleware**: Encrypts/decrypts session cookies
4. **CORS**: Validates cross-origin requests
5. **SecurityHeaders**: Adds security headers
6. **SeoMetadata**: Injects SEO metadata

### Protected Routes

Routes requiring authentication use the `AuthenticatedSession` extractor:

```rust
pub async fn maps_rebuild(session: AuthenticatedSession) -> Result<HttpResponse> {
    // This endpoint only executes if JWT is valid
    // Invalid/missing JWT returns 401 Unauthorized
}
```

## Consequences

### Positive

- **No Session Queries**: Authentication check requires only cryptographic verification, no database lookups
- **Horizontal Scalability**: Any server can validate any token using just the secret key
- **Fast Validation**: HMAC verification is computationally cheap (microseconds)
- **Standard Approach**: JWT is industry-standard; developers are familiar with it
- **Cryptographically Secure**: Signature cannot be forged without the secret key
- **Prevents Tampering**: Any modification of claims invalidates the signature
- **Automatic Expiration**: `exp` claim makes tokens automatically invalid after 60 minutes
- **Immutable Claims**: User role cannot be escalated by modifying the token

### Negative

- **No Early Revocation**: Tokens cannot be revoked before expiration (60-minute TTL)
- **Secret Key Risk**: If `JWT_SECRET` is compromised, all tokens are compromised
- **Token Size**: JWT tokens are larger than simple session IDs (typically 200-500 bytes)
- **Client Responsibility**: Token must be sent with every request; client cannot forget

### Neutral

- **TTL Trade-off**: 60-minute expiration is a balance between security and UX
- **No Refresh Token Logic**: Currently using simple expiration; refresh tokens could be added later
- **Stateless**: Server doesn't store any session state (can be viewed as pro or con)

## Risk Mitigation

### Risk: Secret Key Compromise

- Store `JWT_SECRET` in environment variables, never in code
- Use strong random secret (minimum 256 bits)
- Rotate secret periodically in production

### Risk: Token Interception

- `Secure` flag forces HTTPS in production
- `HttpOnly` flag prevents JavaScript access
- TLS encryption protects token in transit

### Risk: CSRF Attack

- `SameSite=Strict` prevents cross-site cookie sending
- Cookie only sent to same-origin requests

### Risk: XSS Attack

- `HttpOnly` flag prevents JavaScript from reading cookie
- CSP (Content-Security-Policy) header restricts inline scripts

### Risk: Token Tampering

- HMAC signature validates token integrity
- Any payload modification invalidates signature
- Role escalation attempts are cryptographically blocked

## Testing Strategy

### Unit Tests

- Verify token signature validation
- Verify token expiration enforcement
- Verify issuer validation
- Verify claim extraction

### Integration Tests

- Test authenticated endpoint access
- Test missing/invalid token rejection
- Test expired token rejection
- Test token tampering detection

### Security Tests

- Attempt token forgery (should fail)
- Attempt token tampering (should fail)
- Attempt to add fake claims (should fail)
- Test CSRF protection
- Test XSS protection

## Production Requirements

Before deploying to production:

1. Set `COOKIE_SECURE=1` (requires HTTPS setup)
2. Generate strong `JWT_SECRET` (minimum 256 bits of entropy)
3. Set `CORS_ALLOWED_ORIGINS` to restrict cross-origin requests
4. Enable HTTPS/TLS on the server
5. Configure appropriate `JWT_TTL_MINUTES` (default 60 is reasonable)
6. Monitor logs for failed authentication attempts
7. Set up alerts for unusual authentication patterns

## Future Considerations

### Refresh Tokens

Could implement refresh token pattern to allow longer-lived sessions without increasing JWT expiration:

- Short-lived access token (15 minutes)
- Longer-lived refresh token (7 days)
- Refresh token stored in database for revocation capability

### Token Blacklist

For cases where early token revocation is critical:

- Maintain a Redis cache of revoked tokens
- Check revocation on every request
- Trade-off: Adds database lookup back, but only for revoked tokens

### Multi-Factor Authentication (MFA)

Currently not implemented; can add:

- MFA flag in JWT claims
- Require MFA re-verification for sensitive operations

## Decision Makers

- **Michael Bruno** - Project Lead & Backend Architecture

## Confirmation Date

December 9, 2025

## Related ADRs

- None yet

## References

- [JWT.io - Introduction to JSON Web Tokens](https://jwt.io/introduction)
- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [jsonwebtoken Rust Crate](https://github.com/Keats/jsonwebtoken)
- [OWASP CSRF Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [OWASP XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [SameSite Cookie Attribute](https://tools.ietf.org/html/draft-west-first-party-cookies)
