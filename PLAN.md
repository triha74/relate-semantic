# Plan: build relate-semantic

`DESIGN.md` is the contract. This plan builds it the way relate-flow was
built: the same crate split, the same error envelope, the same test layers,
the same documentation topics at the same depth, and the same gosonic CI. Where
relate-flow already has a file, relate-semantic gets the matching file.

## Repository layout

```text
relate-semantic/
  AGENTS.md
  Cargo.toml                  workspace, relate = { git, tag = "v1.0.1" }
  Cargo.lock
  DESIGN.md
  Makefile
  README.md
  rust-toolchain.toml         1.88.0, rustfmt, clippy
  .gitignore                  /target, .relate-semantic, .logs/, *.rl
  .github/
    actions/gosonic/action.yml
    workflows/ci.yml
  .sonic/
    ci.yml
    docker/sonic-rust.Dockerfile
  scripts/sonic-cache-sync.sh
  crates/
    relate-semantic-core/
      src/{lib,error,clock,slug,model,limits,recall,store,ops}.rs
      tests/{smoke,errors,lifecycle,recall_logic,scopes,generations,
             prune,concurrency,invariants,model_check,invoice_logic}.rs
    relate-semantic/
      src/{main,cli,output,prompt,skill}.rs
      src/skill/bundle/SKILL.md
      src/skill/bundle/references/{what-to-remember,errors,invoice,scenarios}.md
      tests/common/mod.rs
      tests/cmd_{scopes,recall,remember,correct,forget,show,log,prune,skill}.rs
      tests/{matrix,docs_sync,fix_commands}.rs
      tests/scenario_{invoice,incident,onboarding,lifecycle}.rs
      tests/snapshots/
  docs/
    architecture.md
    cli.md
    from-a-task.md
    storage.md
    scenarios/{invoice,incident,onboarding}.md
    adr/0001 … 0015
  examples/
    bin.sh
    run-scenario.sh
    invoice/{README.md,seed.sh,run-*.sh}
    incident/{README.md,seed.sh,run-*.sh}
    onboarding/{README.md,seed.sh,run-*.sh}
```

`seed.sh` plays the part of relate-flow's `plan.sh`: it is sourced by each run
script, picks the store, and writes the memories that every case in that
scenario starts from. A scenario with no shared memory starts empty.

## Library: relate-semantic-core

The library owns every rule. The CLI parses flags, calls one method, prints one
object, and exits. It holds no rule.

| Module | Holds |
| --- | --- |
| `error.rs` | `MemoryError`, `Problem`, `ERROR_CODES`. Same shape as relate-flow's `FlowError`. |
| `clock.rs` | `Clock`, `SystemClock`, `FixedClock`, `ManualClock`. Second-precision RFC3339 UTC. |
| `slug.rs` | The slug rule shared by keys, scopes, subjects, and agents. |
| `limits.rs` | 600-character limit on `claim` and `because`, counted in Unicode scalar values. `--limit` default 5, maximum 20. |
| `model.rs` | `Memory`, `Generation`, `Write` values, and the JSON for each. |
| `recall.rs` | The recall decision as a pure function of live memories and the query. No connection. |
| `store.rs` | Load the memory graph, apply a `Write` list, refuse a foreign store, retry `Locked` for about two seconds. |
| `ops.rs` | `Memory::{remember, correct, forget, recall, show, log, prune, scopes}`. One transaction each. |

`Memory::transact` mirrors `Flow::transact`: connect, begin, load, run the rule,
apply writes, commit. A refusal rolls back. Relate's `Conflict` at commit
becomes exit 5, `conflict`, `retryable: true`.

Recall being pure is the decision that makes the strongest tests possible: the
five actions and their order are tested as a table with no database, and the
same function is used by the model check.

## Error reporting

The envelope is relate-flow's, unchanged:

```json
{"error":{"code":"key_taken","message":"widget-price is already live in org","fix":"relate-semantic correct widget-price --scope org --claim TEXT --because TEXT","retryable":false}}
```

- `MemoryError { code, message, fix, retryable, errors, exit }` with the same
  constructors: `usage` (2), `refusal` (4), `conflict` (5), `store` (1),
  `busy` (5, `store_busy`, retryable).
- `Problem { code, message, fix, field }`. relate-flow's `step` becomes
  `field`, which names the flag at fault (`claim`, `about`, `scope`).
- `with_problems` puts the first problem in `code`, `message`, and `fix`, and
  lists every problem in `errors`. A `remember` with a blank claim, a repeated
  subject, and a bad scope slug returns all three in one refusal.
- clap is parsed with `try_parse`. A usage error is the same envelope with
  exit 2. Its `fix` is the usage line of that verb.
- `--help` and `--version` stay plain text and exit 0.

The rules for `fix`:

1. When the next step is a command, `fix` is that command, starting with
   `relate-semantic`, filled in with the key, scope, and subjects from the
   refused call. The only placeholders are values the agent has to write:
   `TEXT`, `SUBJ`, `DURATION`.
2. When the next step is not a command (a store path, a busy store), `fix` is
   one imperative sentence.
3. `message` names the value that was refused, quoted the way the agent typed
   it.

The full code list goes in `ERROR_CODES`. Starting set: `usage`, `store`,
`store_busy`, `conflict`, `foreign_store`, `bad_slug`, `empty_field`,
`too_long`, `duplicate_value`, `key_taken`, `no_memory`, `unknown_agent`,
`no_agent`, `foreign_skill`, `skill_drift`, `skill_io`.

## Test harness

The test suite is meant to stop a build that ships a behavior the docs don't
describe, or an error that doesn't tell the agent what to do. It has seven
layers. The first four match relate-flow. The last three go further.

### 1. Library rules

One file per rule area under `crates/relate-semantic-core/tests/`.

| File | Covers |
| --- | --- |
| `smoke.rs` | Open `:memory:`, remember, recall, show. |
| `lifecycle.rs` | remember → correct → forget → remember again, with generations 1 to 4. |
| `recall_logic.rs` | The decision order as a table: every pair of competing actions, including `conflict` beating `narrow`, `narrow` beating `stale`, and `stale` beating `use`. |
| `scopes.rs` | Scope order, a key live only in a later scope, no `--scope` across many scopes, an unknown scope. |
| `generations.rs` | `SUPERSEDES` chain, tombstones, `about` carried over and replaced, a repeated correct writing nothing. |
| `prune.rs` | Prune one scope, leave the others, prune an empty scope. |
| `errors.rs` | Every refusal from the library, with its code, exit, and `fix`. |
| `invoice_logic.rs` | The invoice memories end to end against the library, no binary. |
| `concurrency.rs` | Two `correct` calls on one key and scope: one wins, the other gets a retryable `conflict`, and exactly one `LIVE` link remains. Writes to two different keys both commit. Run eight times with a barrier, like relate-flow. |

### 2. Properties

`invariants.rs` uses proptest over random sequences of remember, correct,
forget, and prune on a small alphabet of keys, scopes, and subjects. After
every sequence:

- At most one live generation per key and scope.
- Generations for a key and scope run 1, 2, 3 with no gap.
- `log` length equals the number of admitted writes.
- A forgotten key is never returned by `recall` or `show`.
- With `--scope a --scope b`, a key live in both always returns the claim in
  `a`.
- Repeating any admitted write writes nothing and returns the same object.
- A claim of 600 characters is admitted. 601 is `too_long`. Multi-byte
  characters count once.

### 3. Model check

`model_check.rs` runs the same random sequences against a small reference
model: a `BTreeMap<(key, scope), Vec<Generation>>` with the rules written in
the most direct way possible. Every operation's JSON output and every
recall's `action` must match between the library and the model. This catches
an ordering or tie-break bug that a single invariant would miss.

### 4. CLI, one file per verb

`crates/relate-semantic/tests/cmd_*.rs`, with a shared `common/mod.rs` copied from
relate-flow's harness:

- `Store::new()` owns a `TempDir`.
- `store.run(args)` asserts exit 0 and parses the one JSON line.
- `store.fail(args, code)` asserts the exit code and parses the error.
- `redact()` replaces `at` with `"[time]"`.
- `RELATE_SEMANTIC_NOW` is set on every command so output is deterministic.
- insta snapshots for the happy path of each verb, under `tests/snapshots/`.

Each `cmd_*` file covers its verb's success shape, its repeated-call
behavior, every error row in its `docs/cli.md` table, `-d` before and after the
verb, `RELATE_SEMANTIC_DATABASE`, `RELATE_SEMANTIC_AGENT`, and `--output text` or
the table output where the verb has one.

### 5. The matrix

`matrix.rs` matches relate-flow's: it fails when a verb, an `ERROR_CODES`
entry, or a recall action (`use`, `none`, `narrow`, `conflict`, `stale`) is
not named by any test.

### 6. Docs stay in sync

`docs_sync.rs` reads the markdown and fails on drift:

- Every `ERROR_CODES` entry has a row in `docs/cli.md` and in the skill's
  `references/errors.md`. Every code in those tables is in `ERROR_CODES`.
- Every verb in the `docs/cli.md` synopsis is a clap subcommand, and every
  subcommand is documented.
- Every flag clap knows for a verb appears in that verb's section of
  `docs/cli.md`.
- Every ` ```json ` block in `docs/cli.md`, `DESIGN.md`, and the skill bundle
  parses.
- The exit-code table in `docs/cli.md` matches the exits in `error.rs`.

### 7. Every fix is a working command

`fix_commands.rs` triggers each refusal through the binary, takes `fix`, and
when it starts with `relate-semantic`, replaces each placeholder (`TEXT`, `SUBJ`,
`DURATION`) with a valid value and runs it against the same store. The
command must parse. For the codes where the fix is the whole repair
(`key_taken` → `correct`, `no_memory` → `remember`, `conflict` → the same
command), it must also exit 0. A refusal whose fix is a sentence is listed in
the test by code, so a new one has to be added there on purpose.

### Scenarios and examples

`scenario_invoice.rs`, `scenario_incident.rs`, `scenario_onboarding.rs`, and
`scenario_lifecycle.rs` drive the binary through the same commands as the
example scripts and assert the full JSON at each step. The shell scripts are
not cargo tests. The gosonic `examples` stage runs them.

### Gate

`cargo llvm-cov --workspace --fail-under-lines 95`, as in relate-flow.

## Example scenarios, aligned with relate-flow

Each relate-flow run script gets a relate-semantic script with the same name. The
memory script is what the agent recalls and remembers while that flow case
runs, using the same company, case names, notes, and actors. A comment above
each block names the flow step it goes with:

```bash
# verify-discounts
mem recall --about northwind --about discounts --scope case --scope org
```

`examples/bin.sh` mirrors relate-flow's: it resolves `RELATE_SEMANTIC_BIN`
(default `target/release/relate-semantic`), picks a store with `example_store`,
and defines `mem` (which prints the command on stderr when
`RELATE_SEMANTIC_VERBOSE=1` and adds `--output text` to `recall` when
`RELATE_SEMANTIC_OUTPUT=text`), plus `advance DURATION` to move
`RELATE_SEMANTIC_NOW`, and `refused CODE ARGS…` to show a refusal and check its
code without stopping `set -e`.

`examples/run-scenario.sh` matches relate-flow's flags (`-v`, `-text`, a
scenario name, optional script names). It reads `# expect: ACTION` from each
script header and checks the `action` of the last JSON line. Across the 15
scripts, every recall action is the last action of at least one script.

### Invoice

The flow case is Northwind INV-1042 under NW-2024. The shared memory is the
agreement and how to handle a shortfall.

| Script | What it shows | Expect |
| --- | --- | --- |
| `run-discount-missing.sh` | `verify-discounts` recalls nothing, remembers `widget-price` in `org`. After `human-review` approves $9,000, remembers `shortfall-notice`. A second case recalls both. | `use` |
| `run-discount-rejected.sh` | The rejection reason belongs to this case only, so it goes in the case scope. The case scope is pruned, and a recall for that key returns nothing. | `none` |
| `run-discount-correct.sh` | `seed.sh` already holds `widget-price`. The agent uses it without reading the agreement. Repeating the same `remember` returns the original object. | `use` |
| `run-vendor-rejected.sh` | `verify-vendor` remembers that Northwind is not on the approved list. The person approves, and the agent corrects `vendor-status`. `log` shows both generations. | `use` |
| `run-missing-agreement.sh` | NW-2024 can't be found. A phone price is stored in the case scope, the agent tries `remember` in `org` and gets `key_taken`, and a recall with no `--scope` returns `conflict`. | `conflict` |

### Incident

The flow case is a checkout outage with `oncall`, `ic`, and `comms`. The shared
memory is runbooks and known causes.

| Script | What it shows | Expect |
| --- | --- | --- |
| `run-sev1-resolved.sh` | `mitigate` recalls no runbook. Draining the checkout pool works, and `postmortem` remembers it with the incident as the source. | `use` |
| `run-sev2-unresolved.sh` | The runbook comes back, does not work, and `vendor-escalation` runs. The agent corrects the runbook to say when to page the vendor. | `use` |
| `run-sev3.sh` | The runbook is 45 days old. `recall --fresh 30d` returns `stale`. | `stale` |
| `run-retry.sh` | A case note from the failed first attempt is forgotten. Forgetting it again returns `"forgotten": false`. | `none` |
| `run-replan.sh` | Many services share the `checkout` subject. A broad recall returns `narrow`, and the agent narrows by key. | `narrow` |

### Onboarding

The flow case is an account application with `intake`, `kyc`, `screening`, and
`compliance`. The shared memory is compliance rulings that apply to later
applicants.

| Script | What it shows | Expect |
| --- | --- | --- |
| `run-clear.sh` | `risk-score` remembers the low-risk threshold the analyst used. The next applicant recalls it. | `use` |
| `run-mismatch-approved.sh` | `compliance-review` approves a transliterated name. The ruling is remembered in `org`, and the next mismatch recalls it. | `use` |
| `run-hit-declined.sh` | A sanctions hit on an entity is remembered in `org`, so a later application from the same entity recalls it. | `use` |
| `run-blocked.sh` | The document can't be read. The agent tries to paste it as a claim and gets `too_long`, then `bad_slug` on a subject, then writes the fact. | `use` |
| `run-reopen-donewhen.sh` | A ruling's subjects change. `correct --about` replaces them, and a recall by the old subject returns nothing. | `none` |

`docs/scenarios/{invoice,incident,onboarding}.md` walk each scenario at the
same depth as relate-flow's, showing the flow step and the memory command side
by side.

## Documentation

Same topics as relate-flow, at the same depth.

| File | Content |
| --- | --- |
| `README.md` | What it is, a quick start, the store and env vars, exit codes, how to run examples, the skill, the docs list, `make` targets, CI. |
| `DESIGN.md` | The product contract. Already drafted. Updated with every behavior change. |
| `AGENTS.md` | The contract for agents changing this repo, in relate-flow's shape. Rules live in the library. Every verb, flag, and code needs a CLI test and a matrix row. Docs and skill bundle change in the same change. Relate stays a git tag. |
| `docs/cli.md` | The full command reference an agent works from. |
| `docs/architecture.md` | The two crates, one transaction, the pure recall, what is stored versus computed, the clock, the skill. |
| `docs/storage.md` | Labels, properties, relationships, `LIVE`, writes, `foreign_store`. |
| `docs/from-a-task.md` | One agent across two Northwind invoices, from `skill install` to a corrected agreement, using the real output of the binary. |
| `docs/scenarios/*.md` | One page per example scenario. |
| `docs/adr/` | The decisions, listed below. |
| Skill bundle | `SKILL.md`, `references/what-to-remember.md` (relate-flow's `designing-a-workflow.md`), `references/errors.md`, `references/invoice.md`, `references/scenarios.md`. |

### docs/cli.md

Written for an agent that has only this page. It follows relate-flow's
`docs/cli.md` exactly and adds what an agent needs to recover on its own:

- Synopsis of every form, then the slug rule, the 600-character rule, and the
  subject AND rule.
- Options table (`-d`, `-h`, `-V`), environment table
  (`RELATE_SEMANTIC_DATABASE`, `RELATE_SEMANTIC_AGENT`, `RELATE_SEMANTIC_NOW`), exit
  table, and the error object with `errors` and `field`.
- One section per verb, in this order: `recall`, `remember`, `correct`,
  `forget`, `show`, `log`, `prune`, the no-verb scope listing, `skill`. Each
  has a synopsis, one paragraph on what it does and what a repeat does, a
  flag table, a sample command, the stdout object with every field, and a
  code table with `Code`, `Exit`, `When`, and the `fix` it returns.
- For `recall`: the decision order as a numbered list, an action table with
  every field and what the agent does next, one JSON example per action, and
  the exact `--output text` format.

### ADRs

Same numbering and subjects as relate-flow where the subject carries over.

| ADR | Decision |
| --- | --- |
| 0001 | Record architecture decisions. |
| 0002 | Library plus stateless CLI. |
| 0003 | Relate as a git tag, `v1.0.1`. |
| 0004 | The memory graph schema. |
| 0005 | Concurrency: one transaction per write, `LIVE` as the conflict point. |
| 0006 | Recall is a pure function. |
| 0007 | Injectable clock and `RELATE_SEMANTIC_NOW`. |
| 0008 | The JSON envelope on every error path. |
| 0009 | Design clarifications (the open questions below, once settled). |
| 0010 | Testing strategy and the 95% gate, including the model check, doc sync, and fix-command tests. |
| 0011 | CI is gosonic. |
| 0012 | The agent skill ships inside the binary. |
| 0013 | Ordered scopes and the override rule. |
| 0014 | Generations and tombstones instead of updates and deletes. |
| 0015 | Prune is the only delete. |

## CI with gosonic

A copy of relate-flow's setup with the names changed.

- `.sonic/ci.yml`: project `relate-semantic`, runners `docker-cli` and
  `localhost/relate-semantic-sonic-rust:1.88`, the `build-sonic-rust` action,
  stages `lint`, `build`, `test`, `coverage`, `examples`, a `scenario` matrix
  of `invoice`, `incident`, `onboarding`, and suites `smoke-test`,
  `full-test`, `all-examples`. Same cargo cache volumes and target
  subdirectories. `build` writes `target/sonic/release/relate-semantic`.
- `.sonic/docker/sonic-rust.Dockerfile`: identical to relate-flow's (Ubuntu
  24.04, Rust 1.88.0, rustfmt, clippy, llvm-tools, cargo-llvm-cov, python3).
- `.github/workflows/ci.yml`: `sonic-init` builds gosonic from
  `triha74/gosonic` at `36b8e47f575c2436515e3ba5f4819771131e83ff` with
  `CGO_ENABLED=0`, then one job per stage, and `examples` as a
  three-scenario matrix with `fail-fast: false`.
- `.github/actions/gosonic/action.yml` and `scripts/sonic-cache-sync.sh`:
  copied as they are.
- `Makefile`: `build`, `test`, `lint`, `coverage`, `sonic-build`,
  `sonic-smoke`, `sonic-full`, `sonic-examples`, `clean`.

## Order of work

Each phase ends green on `make lint`, `make test`, and `make coverage`, and the
docs for what it adds are in the same change.

| Phase | Work | Done when |
| --- | --- | --- |
| 1. Skeleton | Workspace, toolchain, Makefile, `.sonic`, workflow, Dockerfile, cache script, `AGENTS.md`, ADRs 0001–0003, 0008, 0010, 0011. An empty binary that prints `{"scopes":[]}`. | `gosonic ci run smoke-test` passes locally and on GitHub. |
| 2. Envelope | `error.rs`, `ERROR_CODES`, clap `try_parse`, usage JSON, `output.rs`. Matrix, `docs_sync`, and `fix_commands` tests in place, failing on purpose for codes not written yet. | A usage error is JSON with exit 2. The matrix test runs. |
| 3. Writes | Slug, limits, clock, model, store, `remember`, `correct`, `forget`, with library and CLI tests and snapshots. ADRs 0004, 0005, 0007, 0014. | Every write code has a CLI test and a working fix. |
| 4. Recall | Pure `recall.rs`, the decision table, `--fresh`, `--limit`, `--output text`. ADRs 0006, 0013. | `recall_logic.rs` covers every pair of competing actions. |
| 5. Read and prune | `show`, `log` with table output, `prune`, scope listing. ADR 0015. | Every verb is in the matrix. |
| 6. Properties | `invariants.rs`, `model_check.rs`, `concurrency.rs`. | Seeded proptest runs pass in CI, and a failing seed reproduces locally. |
| 7. Examples | `bin.sh`, `run-scenario.sh`, three `seed.sh`, 15 run scripts, three READMEs, four scenario tests, the `examples` stage. | `gosonic ci run full-test` passes, and every recall action ends at least one script. |
| 8. Docs | `docs/cli.md` complete, `architecture.md`, `storage.md`, `from-a-task.md` from real output, three scenario pages, README. | `docs_sync.rs` passes with no allow-list. |
| 9. Skill | Bundle, `skill install/remove/list/check`, ADR 0012, `cmd_skill.rs`. | `skill check` passes against the built binary. |

## Open questions to settle in ADR 0009

- Subjects are AND in `DESIGN.md`. Keep AND, or add `--any`?
- Is 600 characters right for `claim` and `because`, and is it counted in
  characters, not bytes? The plan assumes characters.
- Does `recall --scope` with a scope that has never been written return
  `none`, or is it refused? The plan assumes `none`.
- Does `correct` on a forgotten key refuse with `no_memory`, or is `remember`
  the only way back? `DESIGN.md` says `remember`.
- Do the examples also run relate-flow? This plan keeps the two binaries
  separate and lines them up by name and comments. A later `with-flow` lane
  could run both binaries side by side, which would make relate-flow a CI
  dependency of relate-semantic.
