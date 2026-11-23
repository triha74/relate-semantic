# Storage

The store is a Relate directory, `./.relate-semantic` unless `-d` or `RELATE_SEMANTIC_DATABASE` names another. Relate Semantic writes three labels: `Memory`, `Scope`, and `Subject`. The same directory may also hold a relate-flow graph (`Plan`, `Step`, `Attempt`). A label neither tool writes is refused with `foreign_store`.

```text
(:Memory)-[:IN]->(:Scope)
(:Memory)-[:ABOUT]->(:Subject)
(:Memory)-[:SUPERSEDES]->(:Memory)
```

## Memory

One node per generation. A generation is never edited after it is written, except that `live` is set to false when a later generation replaces it.

| Property | Type | Meaning |
| --- | --- | --- |
| `slot` | string | `scope/key/generation`. Unique. |
| `key` | string | The key slug. |
| `scope` | string | The scope slug. |
| `generation` | int | 1 for the first write of the key in this scope, then one more per write. |
| `claim` | string or null | The fact. Null on a tombstone. |
| `because` | string | Where the claim came from, or why it changed or was forgotten. |
| `about` | list of strings | The subjects, sorted. Empty on a tombstone. |
| `agent` | string | Who wrote it. |
| `at` | string | RFC 3339 UTC, to the second. |
| `forgotten` | bool | True on a tombstone. |
| `live` | bool | True on the one generation recall can return. At most one per key and scope. |

## Scope and Subject

`Scope {name}` and `Subject {name}` are unique by name. A scope exists while it holds a generation. A subject is shared by every memory that carries it, across scopes. `prune` deletes a scope and every subject no remaining memory uses.

`about` repeats the subject names on the generation so a command loads everything with one query. The `ABOUT` edges are what a direct graph reader would follow.

## Constraints

```text
CREATE CONSTRAINT memory_slot IF NOT EXISTS FOR (m:Memory) REQUIRE m.slot IS UNIQUE
CREATE CONSTRAINT scope_name IF NOT EXISTS FOR (s:Scope) REQUIRE s.name IS UNIQUE
CREATE CONSTRAINT subject_name IF NOT EXISTS FOR (t:Subject) REQUIRE t.name IS UNIQUE
```

They are created on the first write, outside the command's transaction. `memory_slot` is what makes two racing writes to one key conflict. See ADR 0005.

## One key over time

`remember`, `correct`, `forget`, then `remember` again leaves four generations of `widget-price` in `org`:

| generation | claim | live | forgotten | SUPERSEDES |
| --- | --- | --- | --- | --- |
| 1 | NW-2024 price | false | false | |
| 2 | NW-2025 price | false | false | 1 |
| 3 | null | false | true | 2 |
| 4 | NW-2025 price | true | false | 3 |
