# ADR-0007: Injectable clock

Status: Accepted

## Context

Every generation records `at`, and `recall --fresh` compares it to now. Tests and examples need time to stand still or jump forward.

## Decision

The library takes a `Clock`. `SystemClock` is the default, `FixedClock` stands still, and `ManualClock` advances on demand. Times are RFC 3339 UTC with seconds, such as `2026-11-10T18:00:00Z`.

The CLI reads `RELATE_SEMANTIC_NOW` when it is set. An unparseable value is a usage error. The examples move the clock with `advance DURATION`.

## Alternatives considered

- Sleep in tests. Slow and flaky.
- A `--now` flag. It would show up in every agent's command line and docs.

## Consequences

Snapshot tests are deterministic. The variable is documented under Environment in `docs/cli.md` as a testing aid.
