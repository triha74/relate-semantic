# ADR-0013: Scopes are flat, named, and ordered at recall

Status: Accepted

## Context

A fact can hold for the whole organization, for one customer, or for one case. A case override must win over the general rule without erasing it, and a finished case must be removable in one step.

## Decision

Every memory belongs to exactly one scope, a slug such as `org` or `case-inv-1042`. Scopes have no hierarchy. A key is unique within a scope, so the same key can live in several scopes at once.

Recall decides precedence. Without `--scope`, a key live in two scopes is a `conflict`. With `--scope a --scope b`, only those scopes are searched and `a` wins a shared key. A scope exists once it holds a generation. Recalling an unknown scope returns `none`. With no verb, `relate-semantic` lists every scope with its live key count and generation count.

## Alternatives considered

- A fixed hierarchy such as org, customer, case. Real work does not have three levels.
- Precedence stored on the scope. Two agents would disagree about it and edit it back and forth.

## Consequences

The agent states precedence in every recall that needs it, so the decision is visible in the command line. `prune --scope` removes a scope and its whole history.
