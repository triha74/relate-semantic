# ADR-0003: Relate as a git tag

Status: Accepted

## Context

Relate Semantic embeds Relate. Relate Flow pins Relate the same way, and the two products should move together.

## Decision

The workspace depends on `relate = { git = "https://github.com/relatedb/relate", tag = "v1.0.1" }`. Upgrading is a change to that tag and to `Cargo.lock`.

## Alternatives considered

- A path dependency on a local checkout.
- A crates.io release.

## Consequences

CI fetches Relate from GitHub, and the cargo-home cache keeps it between runs. A path dependency is refused in review, as `AGENTS.md` says.
