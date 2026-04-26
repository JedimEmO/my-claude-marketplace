---
name: enforce-robustness
description: Use when the user asks to make code more reliable, add tests, raise coverage, protect against regressions, verify an AI-generated change, build confidence before shipping, create UAT or acceptance tests, add mutation/property/contract tests, or enforce "aggressive trust building" through unit, integration, end-to-end, feature regression, and verification evidence.
---

# Enforce Robustness

Build evidence that the code behaves correctly under realistic use, edge cases, and future edits. Treat tests as a trust-building artifact that should usually be committed with the production change.

## Operating Standard

Default to a high bar unless the user sets a narrower scope:

- Protect critical behavior with executable tests before considering work complete.
- Prefer behavior and invariant coverage over line coverage alone.
- Push coverage toward the practical maximum for changed code; target 100% branch coverage for critical decision logic when feasible.
- Use mutation testing when a mature tool exists for the stack, especially around business rules, parsers, authorization, money, state machines, migrations, and recovery paths.
- Add regression tests for every confirmed bug and every risky edge case discovered while reviewing the change.
- Keep generated tests deterministic, maintainable, and aligned with existing test style.

## Workflow

1. **Map the trust boundary.** Identify the behavior being changed, public interfaces, persistence effects, external calls, concurrency boundaries, and user-visible workflows.
2. **Inventory current evidence.** Locate existing unit, integration, contract, snapshot, browser, UAT, property, fuzz, and regression tests. Run the smallest relevant subset to establish the current state.
3. **Find blind spots.** Compare changed behavior against existing tests. Look for untested branches, failure paths, boundary values, permission states, compatibility cases, migrations, and UI workflows.
4. **Write the missing tests.** Add focused tests in the closest existing test layer. Prefer small unit tests for pure logic, integration tests for boundaries, and UAT/end-to-end tests for user promises.
5. **Add regression protection.** When a bug, edge case, or near-miss is found, create a test that fails for the vulnerable implementation and passes after the fix.
6. **Escalate evidence for high-risk code.** Add property tests, mutation testing, model-based tests, golden fixtures, contract tests, or race/concurrency tests where ordinary examples are too weak.
7. **Run and tighten.** Execute tests, coverage, mutation checks, and lint/build commands that are reasonable for the repo. Fix weak assertions, flaky timing, excessive mocks, and tests that only exercise implementation details.
8. **Report the evidence.** Summarize what was added, what commands passed, what risk remains, and any tool unavailable in the environment.

## Test Selection

Use this table to choose the next test layer:

| Risk | Strong evidence |
| --- | --- |
| Pure business rule, parser, serializer, validator | Unit tests with boundary cases, table tests, property tests |
| Stateful workflow, lifecycle, cache, retry, transaction | Integration tests with real or close test doubles |
| Public API or SDK contract | Contract tests, schema validation, compatibility fixtures |
| UI feature or user promise | UAT/end-to-end tests that follow the user workflow |
| Past bug or production incident | Minimal regression test plus the broader scenario that allowed it |
| Complex branching or critical invariants | Coverage report plus mutation testing |
| Concurrency, async, scheduling, idempotency | Stress tests, deterministic schedulers if available, repeated runs |

## Tooling Heuristics

Prefer the repo's configured commands, then common defaults:

- Rust: `cargo test`, `cargo nextest run`, `cargo llvm-cov`, `cargo mutants`, `proptest`, `quickcheck`, `loom`.
- TypeScript/JavaScript: `npm test`, `vitest`, `jest`, `playwright`, `nyc` or built-in coverage, `stryker`.
- Python: `pytest`, `pytest-cov`, `hypothesis`, `mutmut`, `cosmic-ray`.
- Go: `go test ./...`, `go test -race`, `go test -cover`, fuzz tests with `go test -fuzz`.
- JVM: JUnit, Gradle/Maven test tasks, JaCoCo, PIT mutation testing.

If a tool is not installed or would require network access, state the exact command that would be used and continue with locally available evidence.

## UAT And Feature Regression

For user-facing changes, create at least one test that reads like the user's actual workflow:

- Start from a realistic user state.
- Perform the same action sequence a user or API client would perform.
- Assert the observable result, not just internal calls.
- Include the original bug or requirement wording in the test name only when it improves traceability.

Avoid brittle selectors, sleeps, over-mocked dependencies, and snapshots that are too broad to diagnose.

## Quality Gate

Before finalizing:

- Run the relevant test suite and any new test in isolation.
- Check coverage or mutation score when tooling exists.
- Inspect the final diff for tests that can pass without proving behavior.
- Verify each new test would fail against the old bug or missing behavior when practical.
- Call out residual risk honestly, including untested paths and unavailable tools.

For detailed coverage targets and mutation-testing triage, read `references/evidence-standards.md`.
