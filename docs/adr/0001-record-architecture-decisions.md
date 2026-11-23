# ADR-0001: Record architecture decisions

Status: Accepted

## Context

Relate Semantic is a small product with a strict contract. The reasons behind that contract should outlive the conversation that produced them, the same way they do in relate-flow.

## Decision

Architectural decisions are recorded in `docs/adr/` as numbered markdown files with Status, Context, Decision, Alternatives considered, and Consequences. `DESIGN.md` states the contract. An ADR states why.

## Alternatives considered

- Keep decisions only in `DESIGN.md`.
- Keep decisions in pull request descriptions.

## Consequences

A change to a decision is a new ADR, or an edit that marks the old one superseded. The numbering follows relate-flow where the subject carries over, so the two repositories can be read side by side.
