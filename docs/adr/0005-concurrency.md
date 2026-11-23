# ADR-0005: Concurrency through slot uniqueness

Status: Accepted

## Context

Many agents share one store, each running separate `relate-semantic` processes. Two writes to one key must not both win, and writes to different keys must not block each other. Relate gives one transaction per connection, optimistic write-write conflict detection at commit, and a directory lock held by the opening process.

## Decision

- Each command opens the store, runs one transaction, and closes. A `Locked` open retries for two seconds and then fails with `store_busy` (exit 5, retryable).
- A write reads the key's newest generation N and creates N+1. The unique constraint on `slot` makes two writers that both read N collide when they both create N+1. The loser gets `conflict` (exit 5, retryable) with the fix `run the same command again`.
- Retiring the old generation sets `live = false` on that node, so the conflict is also caught as a write to the same node.
- Edges onto a shared `Scope` or `Subject` node do not conflict, so writes to different keys in one scope both commit.
- `UniquenessViolation` and `Conflict` both mean a race and map to `conflict`.

A spike confirmed each of these behaviors on Relate v1.0.1 before the schema was fixed.

## Alternatives considered

- A global lock node that every write updates. That turns every pair of writes into a conflict.
- Retrying inside the CLI. A retried `correct` might overwrite a claim the agent never read. The agent decides.

## Consequences

The rerun is safe because writes are idempotent: an identical `remember` or `correct` returns the stored generation. Two writes that create the same new scope or subject at the same moment also conflict, and one rerun fixes it. `concurrency.rs` checks this with a barrier.
