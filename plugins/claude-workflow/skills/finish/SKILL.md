---
name: finish
description: Use when the user says they are done, asks to finish a task, wants to verify their work is complete, wants a pre-commit quality check, or asks to validate that changes are ready to ship. Also triggered by phrases like "wrap up", "finalize", "make sure this is done", "are we good?", or "let's finish".
version: 1.0.0
---

# Finish — Pre-Completion Verification Workflow

A checklist-driven workflow that verifies work is actually complete before considering a task done. Run through every step in order. Do not skip steps. If a step fails, fix the issue and re-run that step before proceeding.

## Step 1: Identify What Changed

Run `git diff --stat` and `git diff` to understand the full scope of changes. Run `git status` to catch untracked files that may need to be included. Build a mental model of every file touched and why.

## Step 2: Run Tests

Detect the project's test framework and run the full test suite.

Detection strategy — check in order, use the first match:

| Indicator | Command |
|-----------|---------|
| `Cargo.toml` at root or workspace root | `cargo test --workspace` |
| `package.json` with a `test` script | `npm test` or `yarn test` |
| `pyproject.toml` / `pytest.ini` / `setup.cfg` with pytest | `pytest` |
| `go.mod` | `go test ./...` |
| `justfile` / `Makefile` with a `test` target | `just test` / `make test` |

If no test framework is detected, state this explicitly and skip to Step 3.

If tests fail, fix the failures. Re-run until they pass. Do not proceed with failing tests.

## Step 3: Build and Lint

Run the project's build and lint tooling to catch compilation errors and style issues.

| Language | Command |
|----------|---------|
| Rust | `cargo clippy --workspace --all-targets -- -D warnings` |
| Node/TS | `npm run lint` (if script exists), `npx tsc --noEmit` (if tsconfig.json exists) |
| Python | `ruff check .` or `flake8` (whichever is configured) |
| Go | `go vet ./...` |

If a `justfile` or `Makefile` has a `check` or `lint` target, prefer that.

Fix any issues found. Re-run until clean.

## Step 4: Invoke /simplify

Run the `/simplify` skill. This reviews the changed code for reuse opportunities, code quality, and efficiency. Follow its recommendations and apply fixes.

IMPORTANT: Actually invoke `/simplify` as a slash command. Do not replicate its behavior manually.

## Step 5: Diff Review

Perform a thorough review of the final diff (`git diff` for unstaged, `git diff --cached` for staged). Check for:

- **Correctness:** Does the change do what was intended? Are there edge cases?
- **Leftovers:** Debug prints, TODO comments that should be resolved, commented-out code, hardcoded values that should be configurable.
- **Naming:** Are new functions, variables, and types named clearly?
- **Error handling:** Are errors handled, not swallowed? Are error messages useful?
- **Security:** No secrets, credentials, or API keys in the diff. No injection vectors. No path traversal.
- **Completeness:** If a new public API was added, is it documented? If behavior changed, are docs updated?

If issues are found, fix them. Re-run Steps 2–3 if the fixes are non-trivial.

## Step 6: Summary

Report what was verified:

- [ ] Tests pass — name the command run and result
- [ ] Build/lint clean — name the command run and result
- [ ] /simplify applied — note any changes made
- [ ] Diff reviewed — note any issues found and fixed
- [ ] No secrets or debug artifacts in the diff

State clearly: **"Task verified complete"** or **"Task has unresolved issues:"** followed by what remains.

## Related Skills

For qualitative code analysis beyond this checklist, see **soft-harness-create** and **soft-harness-run**.
