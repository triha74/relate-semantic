# ADR-0012: The agent skill ships inside the binary

Status: Accepted

## Context

An agent uses relate-semantic well only if it knows the loop: recall before acting, follow the action, remember what it learned, and correct or forget what turned out wrong. relate-flow solves the same problem with a skill that `relate-flow skill install` writes into each agent's skills directory. A skill read from the source tree at install time would drift from the binary that runs the commands.

## Decision

The bundle under `crates/relate-semantic/src/skill/bundle/` is compiled in with `include_str!`. It holds `SKILL.md` plus `references/what-to-remember.md`, `references/errors.md`, `references/invoice.md`, and `references/scenarios.md`. `{{VERSION}}` is replaced with the crate version when the files are written.

`relate-semantic skill install|remove|list|check [--agent NAME] [--force]` knows the same 11 agents as relate-flow: claude-code, cursor, windsurf, copilot, antigravity, gemini-cli, cline, codex, pi, opencode, and junie. Each agent has a detect directory and a skills directory. Paths start with `~` or `$XDG_CONFIG_HOME`. Without `--agent`, every agent whose detect directory exists is used, and no detected agent is `no_agent`.

Every install writes a `.relate-semantic-installed` marker holding the version. A directory has one of four states:

- missing;
- owned, meaning it has the marker;
- legacy, meaning it has no marker but its `SKILL.md` says `name: relate-semantic`;
- foreign, which is anything else.

`install` writes into missing, owned, and legacy directories. It replaces a foreign one only with `--force`. `remove` deletes owned and missing directories, and a legacy one only with `--force`. It never deletes a foreign directory. `check` compares each installed file with the bundle and reports `skill_drift` (exit 4, fix `relate-semantic skill install`) on any difference. A read or write that fails is `skill_io` (exit 1).

## Alternatives considered

- Read the skill from `skills/` in the source tree at install time. A binary without its source tree could not install, and the copy could drift from the binary.
- Copy relate-flow's skill and add a memory section. One skill for two tools would be versioned by neither.
- Let `remove --force` delete any directory. Deleting a skill someone else wrote is not recoverable.

## Consequences

The skill always matches the binary that installed it, and `skill check` shows when an upgrade left an old copy behind. `docs_sync` checks that `references/errors.md` lists every error code with its exit status, so a new code cannot ship undocumented to agents. `cmd_skill.rs` runs every action and every refusal against a temporary `HOME`.
