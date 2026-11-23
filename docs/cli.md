# relate-semantic

```text
relate-semantic [-d PATH]
relate-semantic recall [--about SUBJ]... [--key KEY]... [--scope SCOPE]... [--fresh DURATION] [--limit N] [--output json|text]
relate-semantic remember KEY --claim TEXT --about SUBJ... --scope SCOPE --because TEXT [--agent NAME]
relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT [--about SUBJ]... [--agent NAME]
relate-semantic forget KEY --scope SCOPE --because TEXT [--agent NAME]
relate-semantic show KEY
relate-semantic log KEY [--scope SCOPE] [--json]
relate-semantic prune --scope SCOPE
relate-semantic skill install|remove|list|check [--agent NAME] [--force]
```

One invocation opens the store, runs one transaction, prints one compact JSON object on stdout, and exits. Nothing is written to stderr. `--help` and `--version` print plain text and exit 0. Two exceptions print text: `recall --output text`, and `log` when stdout is a terminal.

A memory is one claim, the fact itself, with a `because` saying where it came from. It lives under a key in one scope and carries one or more subjects. Recall finds memories by subject or key. Every write creates a new generation. Old generations are kept and `log` shows them.

Keys, scopes, subjects, and agent names are slugs: a lowercase letter, then lowercase letters, digits, and hyphens. `widget-price`, `case-inv-1042`, and `nw-2024` are slugs. `Widget Price` and `2024` are not. A refused slug comes back with a suggested slug in the `fix`.

`--claim` and `--because` hold at most 600 characters each, counted as Unicode characters. Write the fact in `--claim` and the source in `--because`. Do not paste documents.

## Options

| Flag | Environment | Default | Meaning |
| --- | --- | --- | --- |
| `-d`, `--database PATH` | `RELATE_SEMANTIC_DATABASE` | `./.relate-semantic` | Relate store. The flag wins. It may come before or after the verb. |
| `-h`, `--help` | | | Plain-text help. |
| `-V`, `--version` | | | Plain-text version. |

With no verb, `relate-semantic` prints every scope in the store with its number of live keys and its number of generations, in name order. A missing store prints `{"scopes":[]}` and is not created.

```json
{"scopes":[{"generations":2,"live":0,"name":"case-inv-1042"},{"generations":2,"live":1,"name":"org"}]}
```

`recall`, `show`, `log`, `prune`, and the scope listing never create a store. `remember`, `correct`, and `forget` create it on first write.

## Environment

| Variable | Read by | Meaning |
| --- | --- | --- |
| `RELATE_SEMANTIC_DATABASE` | every verb | Store path. `--database` wins. |
| `RELATE_SEMANTIC_AGENT` | `remember`, `correct`, `forget` | Who is writing. `--agent` wins. |
| `RELATE_SEMANTIC_NOW` | the CLI only | RFC 3339 clock for tests and examples, such as `2026-11-10T18:00:00Z`. The library does not read it. An unreadable value is a usage error. |

## Exit status

| Exit | When |
| --- | --- |
| 0 | The command ran, including every recall action: `conflict`, `narrow`, `stale`, `use`, and `none`. `--help` and `--version`. |
| 1 | The store could not be opened or written, or a skill directory could not be written. |
| 2 | The flags are wrong. The body is still JSON, with `code` `usage`. |
| 4 | The command was refused. `fix` is the command to run instead. |
| 5 | The store is busy, or two writes raced. Run the same command again. |

## Errors

An error is one object on stdout:

```json
{"error":{"code":"no_memory","fix":"relate-semantic remember widget-price --claim TEXT --about SUBJ --scope case-inv-1042 --because TEXT","message":"widget-price is not live in case-inv-1042","retryable":false}}
```

- `code` is stable. Branch on it, not on `message`.
- `message` says what is wrong, naming the value at fault.
- `fix` is usually a whole command that starts with `relate-semantic` and is filled in from the call that was refused. Every slug in it is already corrected. The only placeholders are `TEXT`, `SUBJ`, `SCOPE`, `KEY`, and `DURATION`. Replace them and run it. A few fixes are a sentence, because no command can repair them: `foreign_store`, `unknown_agent`, `no_agent`, and the agent hint on a usage error.
- `retryable` is true for `conflict` and `store_busy`. Run the same command again.
- A usage error's `fix` is the synopsis of the verb that was called.

A refusal with several problems lists every one in `errors`. Each entry has `code`, `message`, `fix`, and `field`, the flag at fault (`key`, `about`, `scope`, `claim`, `because`, or `agent`). The top-level `code` and `message` are the first problem's. The top-level `fix` repairs every slug and drops every repeated subject at once:

```json
{"error":{"code":"bad_slug","errors":[{"code":"bad_slug","field":"key","fix":"use widget-price as the key","message":"key 'Widget Price' is not a slug"},{"code":"bad_slug","field":"about","fix":"pass --about north-wind","message":"--about 'North Wind' is not a slug"},{"code":"duplicate_value","field":"about","fix":"pass each --about once","message":"--about North Wind is repeated"},{"code":"bad_slug","field":"scope","fix":"pass --scope org","message":"--scope 'Org' is not a slug"},{"code":"empty_field","field":"claim","fix":"write the fact in --claim","message":"--claim is blank"}],"fix":"relate-semantic remember widget-price --claim TEXT --about north-wind --scope org --because TEXT","message":"key 'Widget Price' is not a slug","retryable":false}}
```

Every code the binary returns:

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | A flag is missing, unknown, or unreadable, `recall` has neither `--about` nor `--key`, `--limit` is outside 1 to 20, `--fresh` is not a duration, or a write has no agent. | The verb's synopsis, or the corrected command. |
| `store` | 1 | The store could not be opened, read, or written. | Check the path given to `-d`. |
| `store_busy` | 5 | Another process held the store for more than two seconds. | Run the same command again. |
| `conflict` | 5 | Another write to the same key committed first, or two writes created the same new scope or subject at once. | Run the same command again. |
| `foreign_store` | 4 | The directory holds a label that relate-semantic and relate-flow do not write. | Use another `-d`. |
| `bad_slug` | 4 | A key, scope, subject, or agent is not a slug. | The command with every slug corrected. |
| `empty_field` | 4 | `--claim` or `--because` is blank, or `remember` has no `--about`. | The command with `TEXT` or `SUBJ` in the empty place. |
| `too_long` | 4 | `--claim` or `--because` is over 600 characters. | The command with `TEXT` in place of the long text. |
| `duplicate_value` | 4 | The same `--about`, `--key`, or `--scope` was passed twice. | The command with each value once. |
| `key_taken` | 4 | `remember` on a key that is live in that scope with a different claim, source, or subjects. | The `correct` command for that key. |
| `no_memory` | 4 | `correct` on a key that is not live in the scope, or `show` or `log` on a key with nothing to show. | The `remember` command, or `log` for a forgotten key. |
| `unknown_agent` | 4 | `skill --agent` names an agent relate-semantic does not know. | The list of known agents. |
| `no_agent` | 4 | `skill` found no agent to install for. | Pass `--agent NAME`. |
| `foreign_skill` | 4 | The skill directory was not installed by relate-semantic. | On `install`, `relate-semantic skill install --agent NAME --force`. On `remove`, leave the directory in place. |
| `skill_drift` | 4 | `skill check` found an installed copy that differs from this binary. | `relate-semantic skill install`. |
| `skill_io` | 1 | A skill directory could not be read or written. | Check that the skills directory is writable. |

## Verbs

### recall

```text
relate-semantic recall [--about SUBJ]... [--key KEY]... [--scope SCOPE]... [--fresh DURATION] [--limit N] [--output json|text]
```

Returns the live claims that match, or says why there are none. Call it at the start of a step, before reading the source again.

- `--about SUBJ` repeats. A memory matches when it carries every subject given (AND). There is no OR; run two recalls.
- `--key KEY` repeats. When given, the memory's key must be one of these.
- At least one `--about` or `--key` is required.
- `--scope SCOPE` repeats, in the order that should win. Only those scopes are searched, and when one key is live in several of them, the earliest scope wins. With no `--scope`, every scope is searched and a key live in two scopes is a `conflict`. An unknown scope matches nothing.
- `--fresh DURATION` marks a winning claim older than the duration as stale. It takes humantime durations: `30d`, `12h`, `90m`.
- `--limit N` is the most keys a `use` may return. The default is 5. It must be 1 to 20.
- `--output json` is the default. `--output text` prints the answer as a block to paste into a prompt. The first line is always `Recall: ACTION`. A refusal stays JSON.

The answer has exactly one `action`, decided in this order. The first that applies wins.

| `action` | When | Fields | What to do |
| --- | --- | --- | --- |
| `conflict` | No `--scope` was given and one key is live in two or more scopes. | `key`, `scopes` (sorted), `fix`. | Run `fix` with the scopes reordered so the one that should win comes first. |
| `narrow` | More keys match than `--limit`. | `matched`, `limit`, `keys` of `{key, scope, about}`, `fix`. | Pick keys from `keys`, or add an `--about`. `fix` recalls the first key. |
| `stale` | `--fresh` was given and a winning claim is older than it. | `stale` (keys), `fresh`, `memories`. | Check each stale claim's source. If it changed, `correct` it. The claims are still returned. |
| `use` | One or more claims match. | `memories`, ordered by scope rank, then newest first, then key. | Use the claims as written. Do not re-derive them. |
| `none` | Nothing matched. | `about`, `keys`, `scopes`, echoing the query. | Do the work, then `remember` a fact a later case would need. |

A memory in `memories` has `key`, `scope`, `generation`, `claim`, `because`, `about`, `agent`, and `at`.

```json
{"action":"use","memories":[{"about":["discounts","northwind","nw-2024"],"agent":"invoices","at":"2026-11-10T18:00:00Z","because":"Read from NW-2024 section 3 while checking INV-1042.","claim":"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.","generation":1,"key":"widget-price","scope":"org"}]}
```

```json
{"about":["northwind","discounts"],"action":"none","keys":[],"scopes":["case-inv-1042","org"]}
```

```json
{"action":"conflict","fix":"relate-semantic recall --key widget-price --scope case-inv-1042 --scope org","key":"widget-price","scopes":["case-inv-1042","org"]}
```

```json
{"action":"narrow","fix":"relate-semantic recall --about checkout --key checkout-db","keys":[{"about":["checkout","database"],"key":"checkout-db","scope":"org"},{"about":["checkout","flags"],"key":"checkout-flags","scope":"org"}],"limit":1,"matched":2}
```

`--output text` for a `use`:

```text
Recall: use 1 memory.
Use these claims as written. Do not re-derive them.

widget-price (org, generation 1, 2026-11-10T18:00:00Z, by invoices)
Under NW-2024, widgets are $100, with 10% off a widget line over $5000.
Source: Read from NW-2024 section 3 while checking INV-1042.
```

`--output text` for a `stale`:

```text
Recall: stale widget-price.
Older than 30d: widget-price. Check the source in each, then correct it, or recall without --fresh.

[stale] widget-price (org, generation 2, 2026-11-17T17:00:00Z, by invoices)
Under NW-2025, widgets are $110, with 10% off a widget line over $5000.
Source: NW-2025 replaced NW-2024 on 2026-11-01.
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | Neither `--about` nor `--key`, `--limit` outside 1 to 20, or `--fresh` not a duration. | The recall with `--about SUBJ` added, without `--limit`, or with `--fresh DURATION`. |
| `bad_slug` | 4 | A subject, key, or scope is not a slug. | The recall with every slug corrected. |
| `duplicate_value` | 4 | A subject, key, or scope is repeated. | The recall with each value once. |
| `foreign_store` | 4 | The store holds a label that relate-semantic and relate-flow do not write. | Use another `-d`. |

### remember

```text
relate-semantic remember KEY --claim TEXT --about SUBJ... --scope SCOPE --because TEXT [--agent NAME]
```

Writes a new claim for `KEY` in `SCOPE`. Use it for a fact that a later case would need and would otherwise have to work out again. It writes generation 1, or the next generation after a forgotten one.

- `--claim TEXT` is the fact, at most 600 characters.
- `--about SUBJ` repeats and at least one is required. Subjects are what recall searches by. Use the names a later agent would recall with: the customer, the agreement, the topic.
- `--scope SCOPE` is where the fact holds: `org` for everyone, or a case scope such as `case-inv-1042` for one case.
- `--because TEXT` says where the claim came from, at most 600 characters. Name the document, the case, or the person.
- `--agent NAME` is who is writing. `RELATE_SEMANTIC_AGENT` is the same value. One of them is required.

Repeating the same claim, source, and subjects returns the stored generation and writes nothing, so a retried `remember` is safe. A different claim on a live key is refused with `key_taken`, and `fix` is the `correct` command.

Stdout: `{"key","scope","generation","at"}`.

```json
{"at":"2026-11-10T18:00:00Z","generation":1,"key":"widget-price","scope":"org"}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `--claim`, `--about`, `--scope`, or `--because` is missing, or there is no agent. | The synopsis, or `pass --agent NAME or set RELATE_SEMANTIC_AGENT`. |
| `bad_slug` | 4 | The key, a subject, the scope, or the agent is not a slug. | The command with every slug corrected. |
| `empty_field` | 4 | `--claim` or `--because` is blank. | The command with `TEXT` in that place. |
| `too_long` | 4 | `--claim` or `--because` is over 600 characters. | The command with `TEXT` in that place. |
| `duplicate_value` | 4 | A subject is repeated. | The command with each subject once. |
| `key_taken` | 4 | The key is live in the scope with a different claim. `message` quotes the live claim. | `relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT`, with `--about` when the subjects differ. |
| `conflict` | 5 | Another write to the same key committed first. | Run the same command again. |

### correct

```text
relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT [--about SUBJ]... [--agent NAME]
```

Replaces the live claim for `KEY` in `SCOPE` with generation N+1. The old generation stays in `log`. Use it when a recalled claim turned out wrong or out of date.

- `--claim` and `--because` are required, as for `remember`. Say in `--because` why the claim changed.
- `--about` repeats and replaces the whole subject list. Omitted, the subjects are carried forward. A recall by a subject that was dropped no longer finds the key.
- `--agent NAME` or `RELATE_SEMANTIC_AGENT` is required.

The same claim, source, and subjects as the live generation returns it and writes nothing, so a retried `correct` is safe. The same claim with a new `--because` writes a new generation with a new `at`. That is how a claim that `recall --fresh` called stale is confirmed after checking it. A key that is not live in the scope, including a forgotten one, is refused with `no_memory`, and `fix` is the `remember` command with the subjects given or the last ones the key had.

Stdout: `{"key","scope","generation","at"}`.

```json
{"at":"2026-11-17T17:00:00Z","generation":2,"key":"widget-price","scope":"org"}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `--scope`, `--claim`, or `--because` is missing, or there is no agent. | The synopsis. |
| `bad_slug` | 4 | The key, the scope, a subject, or the agent is not a slug. | The command with every slug corrected. |
| `empty_field` | 4 | `--claim` or `--because` is blank. | The command with `TEXT` in that place. |
| `too_long` | 4 | `--claim` or `--because` is over 600 characters. | The command with `TEXT` in that place. |
| `duplicate_value` | 4 | A subject is repeated. | The command with each subject once. |
| `no_memory` | 4 | The key is not live in the scope. `message` says whether it was forgotten. | `relate-semantic remember KEY --claim TEXT --about … --scope SCOPE --because TEXT`. |
| `conflict` | 5 | Another write to the same key committed first. | Run the same command again. |

### forget

```text
relate-semantic forget KEY --scope SCOPE --because TEXT [--agent NAME]
```

Stops `KEY` being live in `SCOPE` by writing a tombstone generation: no claim, no subjects, and `forgotten` true. The history stays in `log`. Use it when a fact no longer holds and there is nothing to replace it with. To bring the key back, `remember` it; that writes the next generation.

- `--because TEXT` says why, at most 600 characters.
- `--agent NAME` or `RELATE_SEMANTIC_AGENT` is required.

Forgetting a key that is not live writes nothing and returns `"forgotten": false`, so a retried `forget` is safe.

Stdout: `{"key","scope","generation","at","forgotten":true}`, or `{"key","scope","forgotten":false}`.

```json
{"at":"2026-11-19T09:00:00Z","forgotten":true,"generation":2,"key":"widget-price","scope":"case-inv-1042"}
```

```json
{"forgotten":false,"key":"widget-price","scope":"case-inv-1042"}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `--scope` or `--because` is missing, or there is no agent. | The synopsis. |
| `bad_slug` | 4 | The key, the scope, or the agent is not a slug. | The command with every slug corrected. |
| `empty_field` | 4 | `--because` is blank. | The command with `TEXT` in that place. |
| `too_long` | 4 | `--because` is over 600 characters. | The command with `TEXT` in that place. |
| `conflict` | 5 | Another write to the same key committed first. | Run the same command again. |

### show

```text
relate-semantic show KEY
```

Prints the live claim for `KEY` in every scope, in scope name order. Use it to see a key's claims without a subject.

Stdout: `{"key","memories"}`. Each memory has the same fields as in `recall`.

```json
{"key":"widget-price","memories":[{"about":["discounts","northwind","nw-2025"],"agent":"invoices","at":"2026-11-17T17:00:00Z","because":"NW-2025 replaced NW-2024 on 2026-11-01.","claim":"Under NW-2025, widgets are $110, with 10% off a widget line over $5000.","generation":2,"key":"widget-price","scope":"org"}]}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `KEY` is missing. | The synopsis. |
| `bad_slug` | 4 | `KEY` is not a slug. | `relate-semantic show` with the corrected key. |
| `no_memory` | 4 | The key is not live in any scope. | `relate-semantic log KEY` when it has history, otherwise the `remember` command. |

### log

```text
relate-semantic log KEY [--scope SCOPE] [--json]
```

Lists every generation of `KEY`, newest first, capped at 50. `--scope SCOPE` limits the list to one scope. A terminal prints a table with columns `GENERATION`, `SCOPE`, `AGENT`, `AT`, and `CLAIM`, and a tombstone prints `(forgotten)`. Any other stdout, or `--json`, prints JSON.

Stdout: `{"key","generations"}`. Each generation has `key`, `scope`, `generation`, `claim` (null on a tombstone), `because`, `about`, `agent`, `at`, `forgotten`, and `live`.

```json
{"generations":[{"about":["discounts","northwind","nw-2025"],"agent":"invoices","at":"2026-11-17T17:00:00Z","because":"NW-2025 replaced NW-2024 on 2026-11-01.","claim":"Under NW-2025, widgets are $110, with 10% off a widget line over $5000.","forgotten":false,"generation":2,"key":"widget-price","live":true,"scope":"org"},{"about":["discounts","northwind","nw-2024"],"agent":"invoices","at":"2026-11-10T18:00:00Z","because":"Read from NW-2024 section 3 while checking INV-1042.","claim":"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.","forgotten":false,"generation":1,"key":"widget-price","live":false,"scope":"org"}],"key":"widget-price"}
```

```text
GENERATION  SCOPE            AGENT        AT                    CLAIM
2           org              invoices     2026-11-17T17:00:00Z  Under NW-2025, widgets are $110, with 10% off a widget line…
1           org              invoices     2026-11-10T18:00:00Z  Under NW-2024, widgets are $100, with 10% off a widget line…
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `KEY` is missing. | The synopsis. |
| `bad_slug` | 4 | `KEY` or the scope is not a slug. | The command with every slug corrected. |
| `no_memory` | 4 | The key has no generations, or none in that scope. | The `remember` command. |

### prune

```text
relate-semantic prune --scope SCOPE
```

Deletes every generation in `SCOPE`, the scope itself, and every subject no other memory uses. It cannot be undone. Use it when a case is closed and its scope holds nothing a later case needs. A key written to the scope afterwards starts again at generation 1. Pruning an empty or unknown scope reports zeros and writes nothing.

Stdout: `{"pruned","keys","generations"}`.

```json
{"generations":2,"keys":1,"pruned":"case-inv-1042"}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | `--scope` is missing. | The synopsis. |
| `bad_slug` | 4 | The scope is not a slug. | `relate-semantic prune` with the corrected scope. |
| `conflict` | 5 | A write to the scope committed first. | Run the same command again. |

### skill

```text
relate-semantic skill install|remove|list|check [--agent NAME] [--force]
```

Installs the relate-semantic skill, which teaches an agent when to recall and what to remember, into an agent's skills directory. `skill` does not open a store. With no `--agent`, the command applies to every detected agent.

- `install` writes the skill. A directory relate-semantic did not install is refused unless `--force`, which replaces it.
- `remove` deletes the skill relate-semantic installed. A relate-semantic skill that lost its install marker is removed only with `--force`. A directory that is someone else's is never removed, even with `--force`.
- `list` shows where the skill is installed and which version.
- `check` compares every installed copy with this binary.
- `--agent NAME` names one agent: `claude-code`, `cursor`, `windsurf`, `copilot`, `antigravity`, `gemini-cli`, `cline`, `codex`, `pi`, `opencode`, or `junie`.
- `--force` lets `install` replace a directory relate-semantic did not install, and lets `remove` delete a relate-semantic skill with no install marker.

`install` and `remove` print the agents they changed. Removing from an agent that has no copy is skipped.

```json
{"agents":["cursor"],"skill":"relate-semantic","version":"0.1.0"}
```

`list` prints each agent's state: `missing`, `installed`, or `foreign`, with the installed `version`.

```json
{"agents":[{"display":"Cursor","name":"cursor","state":"installed","version":"0.1.0"}],"skill":"relate-semantic"}
```

`check` prints `"drift": false` when every installed copy matches, and refuses with `skill_drift` otherwise. `errors` lists each agent that drifted.

```json
{"drift":false,"skill":"relate-semantic","version":"0.1.0"}
```

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | The action is missing or unknown. | The synopsis. |
| `unknown_agent` | 4 | `--agent` is not a known agent. | The list of known agents. |
| `no_agent` | 4 | No agent was detected. | Pass `--agent NAME`. |
| `foreign_skill` | 4 | The directory was not installed by relate-semantic. | On `install`, `relate-semantic skill install --agent NAME --force`. On `remove`, leave the directory in place. |
| `skill_drift` | 4 | `check` found an installed copy that differs from this binary. | `relate-semantic skill install`. |
| `skill_io` | 1 | A skill directory could not be read or written. | Check that the skills directory is writable. |
