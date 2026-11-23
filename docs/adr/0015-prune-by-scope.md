# ADR-0015: Prune deletes a whole scope

Status: Accepted

## Context

Generations are never deleted by the write verbs, so the store only grows. Case scopes are short-lived: once a case is closed, nobody needs its overrides or their history.

## Decision

`prune --scope SCOPE` deletes every generation in the scope, the scope node, and any subject no other memory uses. It reports `{"pruned","keys","generations"}`. Pruning an empty or unknown scope reports zeros, so the command can be repeated. A key written to the scope afterwards starts again at generation 1. `prune` takes no `--agent` and leaves no record.

## Alternatives considered

- Prune by age. That would delete org-wide facts nobody happened to correct, and it would break `log`.
- Prune one key. `forget` already stops a key being live; deleting a single history invites erasing evidence.
- A tombstone for the scope. It would be one more thing for recall to skip, and it would never be read.

## Consequences

Pruning cannot be undone, and `docs/cli.md` says so. An agent should prune only a scope it created for a case it finished.
