---
name: security-audit
description: Use when the user asks for a security review, vulnerability audit, threat modeling, secure-code analysis, dependency audit, fuzzing, sanitizer checks, API verification, SAST/DAST guidance, security tests, exploit regression tests, auth/authz validation, input sanitization checks, secret scanning, or aggressive trust building for security-sensitive code.
---

# Security Audit

Find security problems and turn the important ones into durable evidence. Work on two tracks: analyze the code directly, and add committed tests or verification tooling that prevents regressions.

## Security Stance

Default to adversarial scrutiny for code that handles identity, permissions, money, secrets, network input, file paths, serialization, command execution, cryptography, plugins, migrations, or multi-tenant data.

Do not stop at a checklist. Trace real data and control flow from untrusted input to sensitive sinks. When a vulnerability is confirmed or plausible enough to protect, add a regression test, fuzz target, sanitizer run, or static-analysis rule where the repo can support it.

## Workflow

1. **Scope the assets.** Identify trust boundaries, attacker-controlled inputs, sensitive data, privileged operations, external services, and deployment assumptions.
2. **Map attack paths.** Follow data from entry points to sinks: database queries, shell commands, filesystem paths, SSRF targets, template rendering, deserialization, redirects, logs, and authorization decisions.
3. **Review controls.** Check authentication, authorization, tenancy isolation, validation, encoding, rate limiting, replay protection, session lifecycle, error handling, secrets handling, and audit logging.
4. **Run available tools.** Prefer configured repo tooling first, then language-standard scanners, dependency audit, secret scanning, fuzzing, sanitizer, and type/lint checks.
5. **Write security evidence.** Add tests that fail on the vulnerable behavior: permission bypass, injection payload, malformed input, path traversal, replay, cross-tenant access, unsafe redirect, panic/DoS, or secret leakage.
6. **Fix and verify.** Patch confirmed issues when in scope. Re-run tests and security tools. Add focused regression coverage near the vulnerable boundary.
7. **Report clearly.** Lead with confirmed findings and severity. Distinguish confirmed vulnerabilities, plausible risks, hardening suggestions, and tools that could not be run.

## Evidence To Add

Choose the strongest practical evidence:

| Risk | Evidence |
| --- | --- |
| Authn/authz bypass | Negative permission tests for every role, tenant, and ownership boundary |
| Injection | Payload tests against SQL, NoSQL, shell, LDAP, template, and expression sinks |
| Path traversal or file exposure | Canonicalization tests with encoded, relative, symlink, and absolute paths |
| SSRF or unsafe outbound calls | URL parser allowlist tests and blocked private-network targets |
| Parser/decoder bugs | Fuzz target, corpus fixtures, malformed input regression tests |
| Memory safety or UB | Sanitizer runs, Miri, fuzzing, bounds tests |
| Crypto/session weakness | Token expiry, replay, rotation, algorithm, nonce, and constant-time comparison tests |
| Secret handling | Secret scan plus tests that logs/errors/responses redact sensitive values |
| API contract drift | Schema validation, OpenAPI checks, consumer contract tests |

## Tooling Heuristics

Use local configuration before introducing new commands:

- Cross-language: `semgrep`, `codeql`, `gitleaks`, `trufflehog`, `osv-scanner`.
- Rust: `cargo audit`, `cargo deny`, `cargo clippy`, `cargo miri`, `cargo fuzz`, sanitizer builds where configured.
- TypeScript/JavaScript: `npm audit`, `pnpm audit`, `yarn npm audit`, `eslint` security rules, `tsc`, `playwright` security regressions.
- Python: `pip-audit`, `bandit`, `ruff`, `pytest`, `hypothesis`.
- Go: `govulncheck`, `gosec`, `go test -race`, `go test -fuzz`.
- Containers/IaC: `trivy`, `grype`, `checkov`, `tfsec`, Kubernetes policy linters.

If a tool is missing or requires network access, do not invent results. State that it was unavailable and name the exact command the user can run.

## Audit Depth

For security-sensitive changes, include at least one direct code-review pass and one executable evidence pass:

- Direct pass: inspect the code path manually and reason about attacker control, preconditions, and impact.
- Evidence pass: add or run a test, fuzz target, sanitizer, scanner, or dependency audit that would catch the issue class.

For ambiguous findings, create a small proof-oriented test or reproduction before labeling it a vulnerability.

## Reporting Format

When reporting findings, use:

- Severity: Critical, High, Medium, Low, or Hardening.
- Location: file and line when available.
- Attack path: input, missing control, sink, and impact.
- Evidence: test/tool/manual reasoning that supports the finding.
- Fix: specific mitigation and regression coverage added or recommended.

If no issues are found, say what was examined and what evidence supports the conclusion. Avoid claiming the system is secure; say what risk remains.

For detailed attack categories and test ideas, read `references/security-test-catalog.md`.
