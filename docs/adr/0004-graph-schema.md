# ADR-0004: Graph schema

Status: Accepted

## Context

A memory has a key, a scope, a claim, a reason, subjects, an author, and a time. A key changes over time and every version must stay readable. Recall filters by subject and scope. Two agents can write at once.

## Decision

The store holds three labels and nothing else.

- `Memory` is one generation. Its properties are `slot`, `key`, `scope`, `generation`, `claim`, `because`, `about`, `agent`, `at`, `forgotten`, and `live`. `slot` is `scope/key/generation`. `about` repeats the subject names so a load needs no join. `claim` is null on a tombstone.
- `Scope {name}` holds every generation of a scope through `(:Memory)-[:IN]->(:Scope)`.
- `Subject {name}` is shared between scopes through `(:Memory)-[:ABOUT]->(:Subject)`.
- `(:Memory)-[:SUPERSEDES]->(:Memory)` links a generation to the one it replaced.

Three unique constraints are created on first write: `Memory.slot`, `Scope.name`, and `Subject.name`. A store that holds any other label is refused with `foreign_store`.

## Alternatives considered

- One mutable node per key with a history list property. Every write to a key would rewrite one big value, and history would be one property nobody can query.
- A `LIVE` edge from the scope to the live generation. Moving the edge deletes and creates relationships on the shared scope node. A `live` property on the generation is local to the key.
- Subjects as a property only. Recall by subject would scan every memory. The edge also lets `prune` find orphan subjects.

## Consequences

A command loads every generation in one query and works on plain Rust values. History is kept until `prune`. `docs/storage.md` documents the schema for anyone reading the graph directly.
