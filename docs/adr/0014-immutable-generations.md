# ADR-0014: Immutable generations

Status: Accepted

## Context

A memory that is wrong must be fixable, and an agent reading a claim must be able to see why it changed and who changed it.

## Decision

A generation is never edited. `remember` writes generation 1, or the next generation after a forgotten one. `correct` writes N+1, carries the subjects forward unless `--about` is given, and links N+1 to N with `SUPERSEDES`. `forget` writes a tombstone: `claim` null, `about` empty, `forgotten` true, and `live` false. The only change to an old generation is `live = false`.

An identical repeat (same claim, source, and subjects) returns the stored generation and writes nothing. A `correct` with the same claim and a new source writes a new generation: it records that someone checked the claim again, and it resets the age that `recall --fresh` measures. A `remember` with a different claim on a live key is refused with `key_taken`, whose fix is the `correct` command.

## Alternatives considered

- Update in place with an audit log on the side. Two sources of truth.
- Let `remember` overwrite. An agent could erase another agent's memory without reading it.

## Consequences

`log` shows the full history, newest first, capped at 50. The store grows with every write until `prune --scope` removes a whole scope.
