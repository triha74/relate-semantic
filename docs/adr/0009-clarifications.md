# ADR-0009: Defaults for questions the design left open

Status: Accepted

## Context

The design left several small behaviors open. Each one is visible to agents, so it is fixed here.

## Decision

- Subjects match with AND. `recall --about a --about b` returns memories about both. There is no `--any`; an agent that wants OR runs two recalls.
- `--claim` and `--because` are limited to 600 characters, counted as Unicode characters. Longer text is `too_long`.
- A recall that names an unknown scope returns `none`. It is not an error, because a scope comes into being with its first memory.
- A forgotten key comes back only through `remember`, which writes the next generation. `correct` on a forgotten key returns `no_memory` with a `remember` fix.
- The examples do not run relate-flow. They show the commands an agent would run inside a relate-flow step.
- With no verb, `relate-semantic` prints `{"scopes":[{"name","live","generations"}]}`.

## Alternatives considered

- OR by default. A recall about `northwind` and `discounts` should not return every Northwind memory.
- Truncating long text. A silently cut claim is worse than a refusal.

## Consequences

`docs/cli.md` states each rule where the verb is documented, and tests pin every one.
