# ADR-0010: Testing strategy and coverage gate

Status: Accepted

## Context

The product is a set of refusals, one live claim per key and scope, and a recall decision with five answers. A green build that misses an error code, lets the docs drift from the binary, or ships a `fix` that does not work would ship a contract the tests do not hold.

## Decision

Seven layers:

1. Library rule tests, one file per area, including a concurrency test that races two writes to one key and scope.
2. proptest invariants over random sequences of writes.
3. A model check that runs the same random sequences against a small reference model and compares every output.
4. CLI tests, one file per verb, with insta snapshots, timestamps redacted, and `RELATE_SEMANTIC_NOW` fixed.
5. A matrix test that fails when a verb, an `ERROR_CODES` entry, or a recall action is not named by a test.
6. `docs_sync.rs`, which fails when the error codes, verbs, flags, synopses, or exit codes in the code disagree with `docs/cli.md` and the skill's `references/errors.md`, or when a JSON example in the docs does not parse.
7. `fix_commands.rs`, which triggers each refusal through the binary, fills in the placeholders in its `fix`, and runs it against the same store.

Scenario tests drive the binary through invoice, incident, onboarding, and lifecycle. `cargo llvm-cov --workspace --fail-under-lines 95` is the gate. Example shell scripts are not a cargo test. The gosonic `examples` stage runs them.

## Alternatives considered

- relate-flow's four layers only.
- Count example scripts as `cargo test`.

## Consequences

A new error code needs a test, a row in two docs, and a working fix, or the build fails. Coverage below 95% fails `make coverage` and the coverage stage.
