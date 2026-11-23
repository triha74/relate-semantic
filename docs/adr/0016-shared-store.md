# ADR-0016: Share a directory with relate-flow

Status: Accepted

Amends [ADR-0004](0004-graph-schema.md).

## Context

Relate Flow and Relate Semantic answer different questions, and an agent often wants both for one case. Separate directories make a crash between a plan report and a remembered fact unrecoverable as one commit, and they make the agent point `-d` at two places. ADR-0004 refused every label other than `Memory`, `Scope`, and `Subject`, which made a relate-flow store look foreign.

## Decision

One directory may hold both graphs. The guard allows six labels: `Plan`, `Step`, `Attempt`, `Memory`, `Scope`, and `Subject`. Any other label is `foreign_store` (exit 4). The fix stays a sentence telling the caller to open another directory with `-d`.

Relate Semantic still writes only `Memory`, `Scope`, and `Subject`, and it still creates only the three uniqueness constraints from ADR-0004. It does not read or write a plan. Relate Flow does not read or write a claim. `SUPERSEDES` is used by both graphs, on different node pairs.

A scratch pad, or any later tool, is not in this set. Its labels stay `foreign_store` until a later ADR adds them.

One process holds the writer lock. The other gets `store_busy`.

## Alternatives considered

- Delete the guard. A directory from an unrelated tool would then be writable.
- Keep separate directories and accept that the two writes cannot commit together.

## Consequences

Point both binaries at the same path. An empty directory and a directory that holds only the other tool's graph both open. Recall on a plan with no claims returns `none`. `docs/storage.md` lists the labels the directory may hold.
