---
name: soft-harness-run
description: Use when the user asks to run a soft harness, check code quality metrics, compare against a quality baseline, view quality regressions, run qualitative tests, or evaluate non-functional code properties. Also triggered by "run the harness", "check quality", "how does this compare to baseline", or "quality report".
version: 1.0.0
---

# Soft Harness — Run and Report

Executes a soft harness defined in `.soft-harness/harness.md`, compares results against the baseline, and reports regressions and improvements.

## Step 1: Load the Harness

Read `.soft-harness/harness.md`. If it does not exist, tell the user and suggest creating one with the **soft-harness-create** skill.

Read `.soft-harness/baseline.md` if it exists. This is the comparison target. If no baseline exists, this run will establish one.

## Step 2: Determine Scope

Based on the **Scope** section in the harness definition:

- **project** — analyze all files in the project root, respecting exclude patterns.
- **directory** — analyze only files under the specified paths, respecting exclude patterns.
- **change** — analyze only files changed since the merge base with the default branch. Use `git merge-base HEAD master` (or `main`) to find the base, then `git diff --name-only <base>` for the file list.

Build the file list using Glob, filtering by scope and exclude patterns.

## Step 3: Execute Each Enabled Check

For each check marked **Enabled: yes** in the harness definition, perform the analysis. All checks are purely analytical — read files, count patterns, scan for violations.

### Check Execution

#### Function Length
- Identify function definitions using language-appropriate patterns: Rust (`fn `), TypeScript/JavaScript (`function `, arrow functions, methods), Python (`def `), Go (`func `).
- Count lines from opening to closing brace/dedent.
- Report functions exceeding the threshold with file, line, name, and length.

#### File Length
- Count lines in each file in scope.
- Report files exceeding the threshold.

#### Nesting Depth
- For brace-delimited languages: count brace nesting depth at each line.
- For indentation-based languages (Python): measure indentation levels directly.
- Report locations exceeding the threshold with file, line, and depth.

#### Parameter Count
- Parse function signatures to count parameters.
- Report functions exceeding the threshold.

#### Dependency Direction
- For each rule, scan files matching the `from` pattern for import/use statements.
- Check if any imports match the denied patterns.
- Report violations with file, line, the offending import, and which rule was violated.

#### Forbidden Imports
- Scan files in scope for imports matching forbidden patterns.
- Report violations.

#### Public API Docs
- Identify public items (Rust: `pub fn/struct/enum/trait`, TS/JS: `export`, Python: non-`_` prefixed).
- Check if each has a doc comment directly above it.
- Report percentage documented and list undocumented items.

#### README Presence
- Check each specified directory for a README.md (case-insensitive).
- Report which directories are missing READMEs.

#### Public Export Count
- Count public exports in scope.
- Report total count and delta from baseline.

#### Naming Conventions
- Scan identifiers against expected patterns for the language.
- Report violations.

#### Near-Duplicate Functions / Copy-Paste Indicators
- Identify functions with very similar structure (parameter count, length, name patterns).
- Look for blocks of code appearing nearly verbatim in multiple locations.
- Report suspected duplicates with locations.

## Step 4: Write Results

Write results to `.soft-harness/results/YYYY-MM-DD-HHMMSS.md` with this structure:

```markdown
# Soft Harness Results — YYYY-MM-DD HH:MM

## Summary

- **Files analyzed:** 47
- **Checks run:** 6
- **Passed:** 4
- **Warnings:** 1
- **Errors:** 1

## Check Results

### Function Length — WARNING

3 functions exceed 50 lines:

| File | Line | Function | Length |
|------|------|----------|--------|
| src/handlers.rs | 142 | process_request | 78 lines |
| src/parser.rs | 55 | parse_expression | 63 lines |
| src/utils.rs | 20 | validate_input | 52 lines |

### Dependency Direction — ERROR

1 violation found:

| File | Line | Import | Rule Violated |
|------|------|--------|---------------|
| src/domain/user.rs | 3 | `use crate::infra::db` | domain must not import from infra |

### Public API Docs — PASSED

85% documented (threshold: 80%)

### File Length — PASSED

No files exceed 300 lines.

### Nesting Depth — PASSED

No locations exceed 4 levels.

### Public Export Count — INFO

42 public exports (baseline: 38, delta: +4)
```

## Step 5: Compare Against Baseline

If `.soft-harness/baseline.md` exists, compare the new results:

- **Regressions:** Any check that was passed and is now warning/error, or any check whose violation count increased.
- **Improvements:** Fewer violations, higher percentages, or checks that moved from warning/error to passed.
- **Unchanged:** No significant difference.

## Step 6: Report

Output a human-readable summary to the user:

```
## Soft Harness Results — 2026-03-28 14:30

**6 checks run** | 4 passed | 1 warning | 1 error

### Regressions (vs baseline)
- function_length: 3 violations (was 1) — WARNING
  - src/handlers.rs:142 process_request (78 lines)
  - src/parser.rs:55 parse_expression (63 lines)
  - src/utils.rs:20 validate_input (52 lines)

### Errors
- dependency_direction: domain imports from infra
  - src/domain/user.rs:3 — `use crate::infra::db`

### Improvements (vs baseline)
- public_api_docs: 85% (was 72%)

### Unchanged
- file_length: passed
- nesting_depth: passed
- readme_presence: passed
```

## Step 7: Next Steps

Based on results:

- If there are **errors**, suggest fixing them immediately.
- If there are **regressions**, highlight which changes likely caused them (cross-reference with `git diff`).
- If the user wants to **update the baseline**, copy the current results file to `.soft-harness/baseline.md`.
- Suggest **committing the results** for historical tracking: `git add .soft-harness/results/`.

## Related Skills

To create or modify a harness definition, see **soft-harness-create**.
For task completion verification, see **finish**.
