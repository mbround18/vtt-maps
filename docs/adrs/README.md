# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the VTT Maps project. ADRs document significant architectural decisions, the context that led to them, alternatives considered, and the consequences of the choices made.

## What is an ADR?

An Architecture Decision Record (ADR) is a document that captures a single, justified design choice that addresses a functionally or non-functionally significant requirement. ADRs help teams:

- Understand **why** decisions were made, not just what was decided
- Evaluate **trade-offs** and alternatives that were considered
- **Onboard** new team members by documenting architectural context
- **Revisit** decisions when requirements or constraints change
- Maintain a **decision log** over the project's lifetime

Learn more at [adr.github.io](https://adr.github.io/)

## ADR Template

This project uses the **MADR (Markdown Architectural Decision Records)** template, which emphasizes:

1. **Context**: What problem or situation led to this decision?
2. **Decision**: What was decided and why? (Y-Statement format)
3. **Considered Options**: What alternatives were evaluated and their trade-offs?
4. **Consequences**: What are the outcomes (positive, negative, neutral)?

See [0000-template-madr.md](./0000-template-madr.md) for the full template.

## Current ADRs

### [ADR-0001: JWT-Based Session Authentication](./0001-jwt-authentication.md)

**Status:** Accepted

**Summary:** Decided to use JWT with HMAC-SHA256 signing for session authentication, stored in secure HTTP-only cookies, to achieve cryptographic verification, XSS/CSRF protection, and stateless scalability.

**Key Trade-off:** Tokens cannot be revoked before expiration (60 minutes) but provides stateless validation without database queries on every request.

**Related Files:**

- `packages/actix-backend/src/auth/jwt.rs` - JWT implementation
- `packages/actix-backend/src/auth/extractor.rs` - Authentication middleware
- `packages/actix-backend/src/hooks/` - Security middleware stack

## How to Create a New ADR

1. **Number**: Use the next sequential number (e.g., 0002, 0003, etc.)
2. **Title**: Use a descriptive title in kebab-case (e.g., `0002-database-migration-strategy.md`)
3. **Template**: Copy the structure from [0000-template-madr.md](./0000-template-madr.md)
4. **Fill in sections**: Complete all sections thoughtfully
5. **Review**: Have at least one other team member review before merging
6. **Status**: Start with `Proposed`, change to `Accepted` after approval

## ADR Lifecycle

```mermaid
Proposed → Accepted → (Deprecated | Superseded by [ADR-XXXX])
```

- **Proposed**: Under discussion, not yet approved
- **Accepted**: Approved and implemented
- **Deprecated**: No longer in use but kept for historical record
- **Superseded**: Replaced by a newer ADR (reference the new one)

## Best Practices

✅ **Do**

- Write in clear, concise language
- Document trade-offs honestly
- Include references and links
- Update related ADRs if they reference this decision
- Keep ADRs relatively short (1-3 pages)
- Use MADR's Y-Statement format for clarity

❌ **Don't**

- Include implementation details (reference the code instead)
- Document every decision (only architecturally significant ones)
- Hide or obscure alternatives or downsides
- Leave ADRs in `Proposed` status indefinitely
- Copy-paste from other projects without adaptation

## Reviewing an ADR

When reviewing an ADR, consider:

1. **Is the context clear?** Can someone unfamiliar understand the problem?
2. **Are the options realistic?** Do the considered alternatives make sense?
3. **Are trade-offs honest?** Are downsides acknowledged?
4. **Is the decision justified?** Does the rationale support the choice?
5. **Are there references?** Links to code, docs, or external resources?
6. **Is it maintainable?** Will this help future developers?

## Related Documentation

- [JWT Security Audit](../JWT_SECURITY_AUDIT.md) - Detailed security analysis of authentication
- [Admin Configuration](../ADMIN_CONFIG.md) - Admin user setup and validation
- [Copilot Instructions](../.github/copilot-instructions.md) - Architecture overview

## Questions?

If you have questions about ADRs or want to discuss architectural decisions, please:

1. Open an issue or discussion in the repository
2. Reference the relevant ADR(s)
3. Propose changes in pull requests that update ADRs

---

**Last Updated:** December 9, 2025

**Maintained By:** Project Architecture Team
