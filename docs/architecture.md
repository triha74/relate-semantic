# Architecture

Relate Semantic is two crates.

`relate-semantic-core` owns every rule. `relate-semantic` is a stateless binary: clap, one JSON line, an exit code. The skill installer lives in the binary because it never touches a store.

```text
agent or harness
    |  argv, env
    v
relate-semantic
    |  Memory::{recall,remember,correct,forget,show,log,prune,scopes}
    v
relate-semantic-core
    |  one transaction: guard, load, rule, surgical writes, commit
    v
relate::Database
```

## One transaction

A read (`recall`, `show`, `log`, the scope listing) connects, checks the store is a relate-semantic store, begins, loads every generation into a `State`, runs a pure function over it, and rolls back.

A write (`remember`, `correct`, `forget`, `prune`) does the same, after making sure the three unique constraints exist, and then applies the list of `Write` values the rule pushed: `Create`, `Retire`, or `Prune`. Each touches only the nodes that command changed. A refusal rolls back before anything is written.

`Error::Conflict` or `Error::UniquenessViolation`, at a statement or at commit, becomes exit 5, `conflict`, retryable, fix "run the same command again". `Error::Locked` on open is retried for about two seconds, then `store_busy`.

A read on a path that does not exist opens `:memory:` instead, so `recall` against a missing store returns `none` and creates nothing.

## Rules over plain values

Every verb is a function over `State` in `ops.rs`. Validation in `validate.rs` collects every problem in one pass, so one refusal lists them all and its `fix` repairs every slug at once. The `fix` strings are built in `fix.rs` from the refused call.

`remember`

1. Validate the key, subjects, scope, claim, source, and agent.
2. If the key is live in the scope with the same claim, source, and sorted subjects, return that generation.
3. If it is live with anything else, refuse `key_taken` with the `correct` command.
4. Otherwise create generation N+1, where N is the newest generation of the key in that scope, or 0.

`correct`

1. Validate. An empty `--about` list means "keep the subjects".
2. If the key is not live in the scope, refuse `no_memory` with the `remember` command.
3. If the claim, source, and subjects are unchanged, return the live generation. A new source alone writes a new generation, which is how a stale claim is confirmed.
4. Otherwise retire the live generation and create N+1 that supersedes it.

`forget` retires the live generation and creates a tombstone N+1. With nothing live it returns `"forgotten": false`.

`prune` deletes the scope's generations, the scope, and orphaned subjects.

## Recall is a pure decision

`decide(live, query, now)` in `recall.rs` sees only the live generations, the query, and the time. It filters by scope, key, and every subject, groups by key, and returns the first action that applies:

1. `conflict`: no `--scope`, and a key is live in more than one scope.
2. `narrow`: more keys than `--limit`.
3. `stale`: `--fresh` given, and a winning claim is older than it.
4. `use`: the claims, by scope rank, then newest, then key.
5. `none`.

With `--scope`, each key resolves to the generation in the earliest listed scope before the limit is counted. `recall_logic.rs` tests every pair of actions without a store. `--output text` is rendered from the same JSON value in the binary.

## Concurrency

Two agents run two processes. Each opens the store, runs one transaction, and exits.

- Two writes to one key both read generation N and both create N+1. The unique constraint on `Memory.slot` (`scope/key/generation`) rejects the second, and that caller gets `conflict`.
- Writes to different keys in the same scope add edges to the shared `Scope` node. Edges do not conflict, so both commit.
- Two writes that create the same new scope or subject at once collide on its unique name. One rerun settles it.

The rerun is safe because writes are idempotent. `concurrency.rs` races threads on one store with a barrier, and `processes.rs` races real processes.

## Clock

The library takes a `Clock`. Production uses `SystemClock`. Tests use `FixedClock` or `ManualClock`. The CLI reads `RELATE_SEMANTIC_NOW` and installs a `FixedClock`. The library does not read the environment.

## Skill

The markdown bundle is compiled in with `include_str!`. `skill install` writes it under the agent's skills directory and leaves `.relate-semantic-installed`. The catalog is the same eleven agents Relate installs for. See ADR 0012.
