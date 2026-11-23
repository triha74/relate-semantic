# ADR-0008: JSON envelope for every error path

Status: Accepted

## Context

An agent has to read every failure the same way, including a mistake in the flags, and it has to know what to do next without reading the docs.

## Decision

The envelope is relate-flow's. clap is parsed with `try_parse`. Usage errors become `{"error":{"code":"usage","message","fix","retryable":false}}` and exit 2. Refusals exit 4. Conflicts and `store_busy` exit 5. Store and skill IO failures exit 1. `--help` and `--version` stay plain text and exit 0.

A refusal with several problems lists every one in `errors`. Each entry has `code`, `message`, `fix`, and `field`, the flag at fault. The top-level `code` and `message` are the first problem's. The top-level `fix` is the whole command corrected: every slug replaced with a suggested slug, a repeated subject dropped, and text the agent must rewrite left as `TEXT`.

A `fix` that names a command starts with `relate-semantic` and is filled in from the refused call. The only placeholders are `TEXT`, `SUBJ`, `SCOPE`, `KEY`, and `DURATION`. A usage error's `fix` is the synopsis of that verb.

## Alternatives considered

- Let clap print its own usage text to stderr.
- Use relate-flow's rule that the top-level `fix` is the first problem's fix.

## Consequences

Agents parse stdout only. A client that reads only `fix` can repair every slug problem in one retry. `fix_commands.rs` runs each refusal's fix to keep this true.
