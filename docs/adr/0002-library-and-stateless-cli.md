# ADR-0002: Library plus stateless CLI

Status: Accepted

## Context

An agent calls the memory from a shell, one command at a time. A harness may embed the same rules in a process. Both must get the same answers.

## Decision

`relate-semantic-core` owns every rule and is the only writer. `relate-semantic` is a stateless binary: it parses flags with clap, calls one library method, prints one JSON object, and exits. The skill installer lives in the binary because it never opens a store.

## Alternatives considered

- A long-running service with an HTTP API.
- Rules in the CLI and a thin storage library.

## Consequences

There is no daemon, no cache between calls, and no state outside the store. Every rule can be tested without spawning the binary. The binary tests check flags, output, and exit codes.
