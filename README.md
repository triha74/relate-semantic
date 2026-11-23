# Relate Semantic

Relate Semantic is semantic memory. It embeds [Relate](https://github.com/relatedb/relate) and keeps the facts agents chose to remember. An agent recalls before it acts, and remembers what a later case would otherwise have to work out again. Every claim carries its source, its author, and its history.

The binary is `relate-semantic`. It is not a vector store and it does not summarize: it stores the sentence the agent wrote, and returns it when the agent asks by subject or key. It is the sibling of relate-flow, which drives the steps a memory is recalled in.

## Quick start

```bash
cargo build -p relate-semantic
export PATH="$PWD/target/debug:$PATH"
export RELATE_SEMANTIC_AGENT=invoices

relate-semantic recall --about northwind --about discounts --scope case-inv-1042 --scope org
relate-semantic remember widget-price \
  --claim 'Under NW-2024, widgets are $100, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2024 --about discounts --scope org \
  --because 'Read from NW-2024 section 3 while checking INV-1042.'
relate-semantic recall --about northwind --about discounts --scope case-inv-1077 --scope org
```

The first recall answers `none`. The second answers `use` with the claim:

```json
{"action":"use","memories":[{"about":["discounts","northwind","nw-2024"],"agent":"invoices","at":"2026-11-10T18:00:00Z","because":"Read from NW-2024 section 3 while checking INV-1042.","claim":"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.","generation":1,"key":"widget-price","scope":"org"}]}
```

The store defaults to `./.relate-semantic`. `-d` and `RELATE_SEMANTIC_DATABASE` select another one. `--agent` falls back to `RELATE_SEMANTIC_AGENT`. `RELATE_SEMANTIC_NOW` fixes the clock for tests and examples.

## The verbs

| Verb | What it does |
| --- | --- |
| `recall` | Returns the live claims by subject or key, or says why not: `conflict`, `narrow`, `stale`, `use`, or `none`. |
| `remember` | Writes a new claim for a key in a scope. |
| `correct` | Replaces a live claim with a new generation. The old one stays in `log`. |
| `forget` | Stops a key being live. The history stays. |
| `show` | The live claim for a key in every scope. |
| `log` | Every generation of a key, newest first. |
| `prune` | Deletes a whole scope when its case is closed. |
| `skill` | Installs the agent skill. |

With no verb, `relate-semantic` lists the scopes in the store.

Every result is one compact JSON object on stdout. Exit 0 is an answer or a successful write, including every recall action. Exit 2 is usage. Exit 4 is a refusal. Exit 5 is a conflict or a busy store; run the same command again. Exit 1 is a store or skill filesystem failure. A refusal's `fix` is the corrected command, ready to run:

```json
{"error":{"code":"key_taken","fix":"relate-semantic correct widget-price --scope org --claim TEXT --because TEXT","message":"widget-price is already live in org: \"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.\"","retryable":false}}
```

## Examples

Three scenarios ship as shell scripts, one per relate-flow example, with the same script names. Each script shows what the agent recalls and remembers during that flow run, and ends with a recall whose action the script header names.

```bash
cargo build --release -p relate-semantic
bash examples/run-scenario.sh invoice
bash examples/run-scenario.sh incident
bash examples/run-scenario.sh onboarding
bash examples/run-scenario.sh -v -text invoice run-missing-agreement
```

The scripts call `target/release/relate-semantic`. Set `RELATE_SEMANTIC_BIN` to use a different binary. `gosonic ci run build` writes a Linux binary to `target/sonic/release/relate-semantic`.

## Skill

`relate-semantic skill install` writes the embedded skill into the agents it detects. The skill teaches an agent when to recall, what is worth remembering, which scope a fact belongs in, and how to read every answer and error. [docs/from-a-task.md](docs/from-a-task.md) follows that loop on one invoice. Changing this repository is the contract in `AGENTS.md`.

## Docs

- [docs/from-a-task.md](docs/from-a-task.md) follows one agent from an invoice to a fact the next case recalls.
- [DESIGN.md](DESIGN.md) is the product contract.
- [docs/cli.md](docs/cli.md) is the command reference.
- [docs/architecture.md](docs/architecture.md) is the crate and transaction layout.
- [docs/storage.md](docs/storage.md) is the Relate graph.
- [docs/scenarios/](docs/scenarios/) walks the three example scenarios.
- [docs/adr/](docs/adr/) records the decisions.

## Develop

```bash
make test
make lint
make coverage
```

CI is gosonic. Stages live in `.sonic/ci.yml`. `make sonic-smoke`, `make sonic-full`, and `make sonic-examples` run them locally when Docker and gosonic are installed.
