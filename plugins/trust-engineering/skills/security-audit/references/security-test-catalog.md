# Security Test Catalog

Use this catalog when designing security regression tests. Select cases relevant to the stack and threat model.

## Authentication And Sessions

- Expired, malformed, missing, and wrong-audience tokens.
- Session fixation, replay, logout invalidation, refresh-token rotation.
- Password reset and email verification token reuse.
- Timing and enumeration differences for login and recovery flows.

## Authorization And Tenancy

- Horizontal access: user A reading, updating, deleting, or listing user B resources.
- Vertical access: low-privilege user calling admin or service-only paths.
- Multi-tenant scoping: query filters, cache keys, background jobs, exports, webhooks.
- Object ownership checked on every mutation, not only on read.

## Input And Injection

- SQL/NoSQL/LDAP/template/expression payloads.
- Command arguments containing spaces, separators, substitutions, and encoded characters.
- HTML, Markdown, CSV, and rich-text payloads that cross rendering contexts.
- Header injection, request smuggling edge cases, and unsafe redirects.

## Files And URLs

- `../`, encoded traversal, absolute paths, symlink traversal, mixed separators.
- MIME confusion, extension spoofing, archive bombs, zip slip.
- SSRF to localhost, metadata services, private ranges, IPv6, DNS rebinding, redirects.

## Reliability As Security

- Oversized payloads, deep nesting, decompression bombs, parser panics.
- Race conditions around authorization, payment, inventory, quotas, and idempotency keys.
- Partial failure that exposes data, repeats side effects, or skips audit logs.

## Secret Handling

- Logs, errors, traces, telemetry, snapshots, and client responses redact secrets.
- Test fixtures do not contain real credentials.
- Config loaders fail closed when required secrets are missing or weak.
