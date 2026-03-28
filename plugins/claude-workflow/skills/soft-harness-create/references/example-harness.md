# Example Soft Harness Definitions

## Rust Workspace Example

A harness for a Rust workspace with a domain-driven architecture (core, api, infra crates).

```markdown
# Soft Harness Definition

## Scope

- **Type:** project
- **Exclude:** **/target/*, **/testutils/**

## Checks

### Function Length
- **Enabled:** yes
- **Threshold:** 50 lines
- **Severity:** warning

### File Length
- **Enabled:** yes
- **Threshold:** 400 lines
- **Severity:** warning

### Nesting Depth
- **Enabled:** yes
- **Threshold:** 4 levels
- **Severity:** warning

### Parameter Count
- **Enabled:** yes
- **Threshold:** 5
- **Severity:** warning

### Dependency Direction
- **Enabled:** yes
- **Severity:** error
- **Rules:**
  - `crates/*-core/src/**` must not import from `crates/*-api/**`, `crates/*-infra/**`, `crates/*-bin/**`
  - `crates/*-api/src/**` must not import from `crates/*-infra/**`

### Forbidden Imports
- **Enabled:** yes
- **Severity:** error
- **Rules:**
  - `crates/*-core/**` must not import `tokio`, `axum`, `diesel`, `sqlx`
  - `crates/*-core/**` must not import `std::fs`, `std::net`

### Public API Docs
- **Enabled:** yes
- **Threshold:** 80%
- **Severity:** warning

### Public Export Count
- **Enabled:** yes
- **Severity:** info
```

**Why these choices:** Rust workspaces benefit strongly from dependency direction enforcement — it's easy for a domain crate to accidentally pull in infrastructure types. The forbidden imports check reinforces this at the module level. Public API docs are important for library crates that other crates depend on. `unwrap` detection is left to clippy (`unwrap_used` lint), so it's not duplicated here.

---

## TypeScript/Node Project Example

A harness for a TypeScript project with a layered architecture (domain, services, api, infrastructure).

```markdown
# Soft Harness Definition

## Scope

- **Type:** directory
- **Paths:** src/
- **Exclude:** **/*.test.ts, **/*.spec.ts, **/node_modules/*, **/dist/*

## Checks

### Function Length
- **Enabled:** yes
- **Threshold:** 40 lines
- **Severity:** warning

### File Length
- **Enabled:** yes
- **Threshold:** 250 lines
- **Severity:** warning

### Nesting Depth
- **Enabled:** yes
- **Threshold:** 4 levels
- **Severity:** warning

### Parameter Count
- **Enabled:** yes
- **Threshold:** 4
- **Severity:** warning

### Dependency Direction
- **Enabled:** yes
- **Severity:** error
- **Rules:**
  - `src/domain/**` must not import from `src/infrastructure/**`, `src/api/**`
  - `src/services/**` must not import from `src/api/**`

### Public API Docs
- **Enabled:** no

### Public Export Count
- **Enabled:** yes
- **Severity:** info

### Naming Conventions
- **Enabled:** yes
- **Severity:** warning
- **Rules:**
  - Functions and variables: camelCase
  - Classes and types: PascalCase
  - Files: kebab-case
```

**Why these choices:** TypeScript projects tend toward shorter functions and files than Rust. Lower thresholds reflect this. Documentation coverage is disabled because TSDoc adoption varies — enable it if the project uses it consistently. Naming conventions are checked because TypeScript projects often mix conventions across contributors.
