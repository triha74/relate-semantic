# Agents changing this repository

`DESIGN.md` is the product contract. A behavior change updates it in the same change. An architectural decision gets an ADR in `docs/adr/` first.

The library in `crates/relate-semantic-core` owns every rule: validation, generations, the live claim, recall, and pruning. The CLI in `crates/relate-semantic` parses flags, prints one JSON object, and exits. It holds no rule. The command is `relate-semantic VERB`. A key, a scope, a subject, and an agent are slugs.

Do not extract, summarize, rank, or embed a claim. Do not decide which of two claims is true. Do not add a query language, a service, or a Cypher surface for agents. Do not read a relate-flow store.

A new verb, flag, error code, or recall action needs a CLI test and a row the command matrix can see. Error codes live in `ERROR_CODES`. A new code also needs a row in `docs/cli.md` and in the skill's `references/errors.md`, and its `fix` must pass `fix_commands.rs`. `docs_sync.rs` fails when the code and the docs disagree.

A `fix` that names a command starts with `relate-semantic` and is filled in from the refused call. The only placeholders are `TEXT`, `SUBJ`, `SCOPE`, `KEY`, and `DURATION`.

CI commands live in `.sonic/ci.yml`. The GitHub workflow only builds gosonic and runs one stage per job. `make lint`, `make test`, and `make coverage` are the local equivalents. Coverage stays at or above 95% lines.

The skill bundle under `crates/relate-semantic/src/skill/bundle/` describes the product the binary ships. A CLI or recall change updates the bundle in the same change. `AGENTS.md` is this contract. The skill is for an agent remembering and recalling, not for an agent changing the repo.

Relate is the git dependency `tag = "v1.0.1"`. Do not replace it with a path dependency.
