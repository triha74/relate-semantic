# ADR-0006: Recall is a pure decision with five actions

Status: Accepted

## Context

An agent at the start of a task needs one answer it can branch on: use these claims, or do something specific first. A plain list of matches leaves the agent to judge ties, staleness, and overload on its own, and every agent judges differently.

## Decision

`decide(live, query, now)` is a pure function in `recall.rs`. The command loads the live generations, calls it, and prints its value. It returns exactly one `action`, checked in this order:

1. `conflict`: no `--scope` was given and one key is live in two or more scopes. The fix lists the key and every scope; the agent reorders the scopes so the one that should win comes first.
2. `narrow`: more keys match than `--limit` (default 5, at most 20). The answer lists the keys, not the claims.
3. `stale`: `--fresh` was given and at least one winning claim is older than it. The answer carries the claims and names the stale keys.
4. `use`: the claims, ordered by scope rank, then newest, then key.
5. `none`: nothing matched. The answer echoes the query.

Subjects match with AND. With `--scope`, the earliest listed scope wins a shared key.

## Alternatives considered

- Return every match and let the agent rank. Agents would ignore conflicts.
- Let narrow beat conflict. The agent would narrow and still hit the conflict.

## Consequences

`recall_logic.rs` tests every pair of actions without a store. `--output text` renders the same value as a prompt block whose first line is `Recall: ACTION`.
