# Evidence Standards

Use these targets as pressure, not bureaucracy. Raise or lower them based on criticality, repo maturity, runtime cost, and user constraints.

## Coverage

- Changed critical decision logic: aim for 100% branch coverage.
- Changed ordinary application code: aim for at least 90% line and branch coverage on touched modules when practical.
- Generated, declarative, UI styling, and framework glue code can use lower coverage if behavior is exercised elsewhere.
- Do not accept coverage that only executes code without assertions tied to behavior.

## Mutation Testing

Prioritize mutation testing for:

- Authorization and tenancy logic.
- Financial calculations, billing, quotas, limits, and permissions.
- Parsers, validators, serializers, migrations, and compatibility code.
- Retry, idempotency, reconciliation, and recovery paths.

Triage surviving mutants:

- Add a test when the mutant changes observable behavior.
- Mark equivalent mutants only when the code is genuinely indistinguishable.
- Consider simplifying code when many equivalent or hard-to-kill mutants appear.

## Regression Tests

A regression test should:

- Fail against the broken behavior.
- Assert the externally meaningful result.
- Be named around behavior, not implementation details.
- Live close to the layer where the bug escaped.

## UAT Coverage

For user-facing features, cover:

- The happy path.
- At least one validation or permission failure.
- The most likely recovery path.
- The workflow state after reload, retry, or navigation when applicable.
