---
name: soft-harness-create
description: Use when the user asks to create a soft harness, set up qualitative tests, measure code quality, track non-functional metrics, create architectural conformance checks, analyze code complexity trends, set up documentation completeness tracking, or wants a quality baseline for a module or project. Also triggered by "soft test", "quality harness", "non-functional test suite", or "quality baseline".
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Soft Harness — Create Qualitative Test Suite

A soft harness measures non-functional qualities of code: complexity, architectural conformance, documentation completeness, API surface consistency, and duplication patterns. Unlike unit tests, soft harnesses do not assert correctness — they assess quality and track it over time.

Soft harness definitions and results live in `.soft-harness/` at the project root. They are intended to be committed to the repository for historical tracking.

## Directory Structure

```
.soft-harness/
├── harness.md                        # Harness definition: which checks to run, thresholds, scope
├── baseline.md                       # The accepted baseline (copied from a results file)
└── results/
    └── YYYY-MM-DD-HHMMSS.md         # Timestamped result snapshots
```

## Step 1: Determine Scope

Ask the user (or infer from context) what the harness should cover:

- **Whole project** — analyze everything under the project root
- **Specific module/directory** — analyze a subtree (e.g., `src/domain/`, `crates/app-core/`)
- **Specific change** — analyze only files changed since a base branch

## Step 2: Select Checks

Choose applicable checks from the catalog below. Not all checks apply to every project — select based on the language, project structure, and what the user cares about.

### Check Catalog

#### Complexity

- **Function length:** Count functions exceeding a line threshold (default: 50). Report the longest functions with file, line number, and name.
- **File length:** Count files exceeding a line threshold (default: 300). Report the longest files.
- **Nesting depth:** Identify deeply nested blocks (default: 4 levels). Use brace counting or indentation analysis depending on language.
- **Parameter count:** Functions with more than N parameters (default: 5).

#### Architectural Conformance

- **Dependency direction:** Define allowed import/dependency directions (e.g., "domain must not import from infra"). Scan `use`/`import`/`require` statements for violations.
- **Layer violations:** Define layers and verify dependencies only flow inward. Configurable per project.
- **Forbidden imports:** Modules or packages that should never be imported in certain scopes (e.g., no `tokio` in domain crate).

#### Documentation

- **Public API docs:** Percentage of public functions/types/modules with doc comments. Language-specific detection (Rust: `///` above `pub`, TS: `/** */` above `export`, Python: docstrings on non-`_` items).
- **README presence:** Check that key directories have README files.

#### API Surface

- **Public export count:** Track the number of public exports. A sudden spike may indicate leaky abstraction. Tracked for trend, not pass/fail.

#### Duplication Patterns

- **Near-duplicate functions:** Identify functions with very similar structure (same parameter count, similar length, similar names). Heuristic, not exact.
- **Copy-paste indicators:** Blocks of code that appear nearly verbatim in multiple locations.

#### Consistency

- **Naming conventions:** Check that function/type names follow the project's conventions (snake_case, CamelCase, etc.).
- **Error handling patterns:** Verify consistent error handling (e.g., all functions in a module use Result, no bare unwrap in library code).

## Step 3: Write the Harness Definition

Create `.soft-harness/harness.md` with this structure:

```markdown
# Soft Harness Definition

## Scope

- **Type:** project | directory | change
- **Paths:** (for directory scope) src/domain/, src/services/
- **Exclude:** **/test*, **/generated*

## Checks

### Function Length
- **Enabled:** yes
- **Threshold:** 50 lines
- **Severity:** warning

### File Length
- **Enabled:** yes
- **Threshold:** 300 lines
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
  - `src/domain/**` must not import from `src/infra/**`, `src/api/**`
  - `src/api/**` must not import from `src/infra/**`

### Public API Docs
- **Enabled:** yes
- **Threshold:** 80%
- **Severity:** warning

### Public Export Count
- **Enabled:** yes
- **Severity:** info (track only)
```

Customize checks based on Steps 1–2. Disable checks that do not apply. Adjust thresholds to the project's current state — the first harness should produce a realistic baseline, not a wall of failures.

Severity levels:
- **info** — track the metric but do not flag it. Useful for trend data.
- **warning** — flag in the report but the harness does not "fail".
- **error** — the harness reports a failure.

## Step 4: Run Initial Baseline

After creating the harness definition, invoke the **soft-harness-run** skill to execute it and produce the initial baseline. The first run's results become `baseline.md`.

## Implementation Notes

All checks are performed by Claude reading and analyzing source files directly — no external tools required. Each check is a pattern of file reading, grepping, and counting that Claude performs when the harness is run.

The harness definition is declarative. The **soft-harness-run** skill interprets it and performs the actual analysis.

## Related Skills

To execute a harness and compare results, see **soft-harness-run**.
For task completion verification, see **finish**.
