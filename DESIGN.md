# Relate Semantic

Relate Semantic is semantic memory, a separate product that embeds Relate. A store holds the
facts agents have chosen to keep. An agent recalls the facts about the
subjects in front of it before it acts, and it writes a fact down when a
later agent should not have to learn it again. Relate Semantic admits each write
or refuses it.

The binary is `relate-semantic`.

Relate Semantic is not a model. It does not read a conversation, extract facts,
summarize, embed, or decide which of two claims is true. The agent asserts a
claim and says where it came from. Relate Semantic decides whether that claim
may be stored, which claim is live, and what a recall returns.

## Goals

- An agent recalls by naming subjects and scopes, and receives one
  instruction: use these claims, there is nothing, narrow the question,
  resolve a conflict, or check a stale claim.
- One fact has one live claim in a scope. A changed fact replaces the old
  claim and keeps it in the history.
- Every claim says where it came from. A write without a source is refused.
- A case can override a shared fact without editing it. Scopes are searched in
  the order the agent names them.
- A claim is short enough to put into a prompt as written.
- The memory survives the process. The store is a Relate database.

## Non-goals

- Storing transcripts, conversations, or step logs. The episode stays where it
  happened. For a plan, that is `relate-flow log`.
- Extracting, rewriting, or summarizing a claim. The agent writes the sentence
  the next agent reads.
- Similarity search, embeddings, confidence scores, or ranking by a model.
- A query language for agents. Cypher is not the agent API.
- Deciding truth. Two scopes that disagree are reported, not merged.
- Reading a relate-flow plan, or writing into relate-flow `inputs`. The two
  binaries share a method in the skill, not a store.
- A long-running service, a REPL, a daemon, hooks, or background compaction.

## Shape

Two pieces live in this repository.

- A library opens a `relate::Database` and implements the verbs below. It is
  the only writer. Every rule in this document is a library rule.
- `relate-semantic` is a stateless CLI over that library. One invocation opens
  the store, performs one admission, prints one JSON object, and exits.

Durability, crash recovery, and concurrent writers are Relate's. A write is one
transaction. Two writes to the same key in the same scope conflict, and the
loser is told to call again. Writes to different keys both succeed. Two writes
that create the same new scope or subject at the same moment also conflict,
and one rerun settles it.

The default store is `./.relate-semantic`, a Relate directory. `:memory:` is
accepted for tests. `remember`, `correct`, and `forget` create a store that
does not exist yet. `recall`, `show`, `log`, `prune`, and the scope listing on
a missing store read it as empty and do not create it.

## A memory

A memory is one claim about one fact, in one scope.

| Field | Rule |
| --- | --- |
| `key` | A slug: a lowercase letter, then lowercase letters, digits, and hyphens. Names the fact, not the claim. `widget-price`, `nw-2024-discount`. |
| `scope` | A slug. Who the fact is for and how long it lives. `inv-1042` lives for one case. `org` lives until it is pruned. |
| `claim` | Non-empty. At most 600 characters. Stored verbatim. The sentence the next agent uses. |
| `because` | Non-empty. At most 600 characters. Stored verbatim. Where the claim came from. |
| `about` | One or more distinct slugs. The subjects a recall finds the memory by. |
| `agent` | A slug. Who wrote this generation. Attribution, not ownership. |
| `generation` | 1 for the first claim. Each `correct` or `forget` adds one. |
| `at` | RFC3339 UTC time of this generation. |

A key and a scope identify a memory. A key may be live in several scopes at
once. That is how a case overrides a shared fact.

The limits are there so that a claim stays a fact. A claim longer than 600
characters is refused with `too_long`. The fix says to write the fact and put
the source in `because`.

Relate Semantic does not reserve a scope name, and it does not know what a case
is. The agent picks the scope. The skill uses the relate-flow execution name
for facts that belong to one case, and `org` for facts that outlive it.

## Writing

```bash
relate-semantic remember widget-price \
  --claim "Under NW-2024, widgets are \$100, with 10% off a widget line over \$5000." \
  --about northwind --about nw-2024 --about discounts \
  --scope org \
  --because "Read from the agreement while checking INV-1042." \
  --agent invoices
```

`remember KEY` writes generation 1 of a new memory. `--claim`, `--about`,
`--scope`, `--because`, and `--agent` are required. `--about` repeats.

Repeating the same `remember`, with the same claim, subjects, and source,
prints the original object and exits 0. The same key and scope with any other
claim is refused with `key_taken`. The fix is `correct`. A live claim is
never overwritten by `remember`.

```bash
relate-semantic correct widget-price --scope org \
  --claim "Under NW-2025, widgets are \$110, with 10% off a widget line over \$5000." \
  --because "NW-2025 replaced NW-2024 on 2026-11-01." \
  --agent invoices
```

`correct KEY` writes the next generation of a live memory. `--claim`,
`--scope`, `--because`, and `--agent` are required. `--about` is optional.
When it is given, it replaces the whole subject list. When it is omitted, the
subjects carry over. A correction with the same claim, source, and subjects as
the live generation exits 0 and writes nothing. The same claim with a new
source writes a new generation: that is how an agent confirms a claim that
`--fresh` called stale. `correct` on a key that is not live in that scope,
including a forgotten one, is `no_memory`. The fix is `remember`.

```bash
relate-semantic forget widget-price --scope inv-1042 \
  --because "The case override no longer applies." --agent invoices
```

`forget KEY` writes a tombstone generation. The memory stops being live in
that scope, and the history stays. `--scope`, `--because`, and `--agent` are
required. Forgetting a key that is not live exits 0 with
`{"forgotten": false}`. A forgotten key may be remembered again. That write is
the next generation, not generation 1.

Each write prints the stored generation:

```json
{ "key": "widget-price", "scope": "org", "generation": 2, "at": "2026-11-17T17:00:00Z" }
```

`forget` prints `{"key","scope","generation","at","forgotten":true}`, or
`{"key","scope","forgotten":false}` when nothing was live.

Writes never change an earlier generation. A correction is a new generation
that `SUPERSEDES` the previous one.

## Recall

```bash
relate-semantic recall --about northwind --about discounts --scope inv-1042 --scope org
```

`recall` prints one object. The agent switches on `action`. `use`, `none`,
`narrow`, `conflict`, and `stale` all exit 0. They are answers.

| Flag | Rule |
| --- | --- |
| `--about SUBJ` | Repeatable. A memory matches when it carries every subject given. |
| `--key KEY` | Repeatable. A memory matches when its key is one of these. Combined with `--about`, both must hold. |
| `--scope SCOPE` | Repeatable and ordered. Only these scopes are searched. The first scope wins for a key live in several. Omitted means every scope, with no order. |
| `--fresh DURATION` | A humantime duration (`30d`, `12h`). A winning claim older than that is stale. |
| `--limit N` | The most claims a `use` returns. The default is 5. More than 20 is a usage error. |
| `--output json\|text` | `json` is the default. `text` prints `use` and `stale` as a prompt. A refusal stays JSON. |

At least one `--about` or `--key` is required. A recall of everything is a
usage error.

Subjects are AND, so adding a subject narrows the answer. To widen, the agent
drops a subject. Relate Semantic never guesses that two subjects are related.

Recall first finds every live memory that matches, then picks one winner per
key, then decides:

1. A matched key is live in more than one scope and no `--scope` was given.
   Return `conflict`. No claims are returned. The agent does not act on a fact
   while two scopes disagree about it.
2. More than `--limit` keys have a winner. Return `narrow` with the keys and
   their subjects, not their claims.
3. `--fresh` was given and at least one winner is older than it. Return
   `stale` with every winner, and list the stale keys.
4. At least one key has a winner. Return `use`.
5. Return `none`.

With `--scope`, the winner for a key is the claim in the earliest listed scope
where that key is live. A key live only in a later scope still wins there.
Claims in `use` are ordered by scope order, then newest `at` first, then key.
A recall that names an unknown scope searches nothing there and can return
`none`. It is not an error.

```json
{
  "action": "use",
  "memories": [
    {
      "key": "widget-price",
      "scope": "org",
      "claim": "Under NW-2024, widgets are $100, with 10% off a widget line over $5000.",
      "because": "Read from the agreement while checking INV-1042.",
      "about": ["discounts", "northwind", "nw-2024"],
      "agent": "invoices",
      "generation": 1,
      "at": "2026-11-10T18:00:00Z"
    }
  ]
}
```

```json
{ "action": "none", "about": ["northwind", "discounts"], "keys": [], "scopes": ["inv-1042", "org"] }
```

`none` echoes the query, so the agent can see what it asked.

```json
{
  "action": "narrow",
  "matched": 9,
  "limit": 5,
  "keys": [{ "key": "widget-price", "scope": "org", "about": ["discounts", "northwind", "nw-2024"] }, "…"],
  "fix": "relate-semantic recall --about northwind --about discounts --key widget-price"
}
```

```json
{
  "action": "conflict",
  "key": "widget-price",
  "scopes": ["inv-1042", "org"],
  "fix": "relate-semantic recall --key widget-price --scope inv-1042 --scope org"
}
```

```json
{ "action": "stale", "stale": ["widget-price"], "fresh": "30d", "memories": ["…"] }
```

What the agent does with each:

| `action` | What the agent does |
| --- | --- |
| `use` | Carry those claims into the work as written. Do not re-derive them. |
| `none` | Do the work. `remember` only a fact a later agent would need. |
| `narrow` | Recall again with a `--key` or another `--about`. |
| `conflict` | Recall again with `--scope` in the order that should win. Do not pick a claim. |
| `stale` | Check the source in `because`, then `correct`, or recall without `--fresh`. |

`--output text` prints a `use` as the claims, each followed by its source and
scope, so the agent can paste it into the step. `stale` prints the same, with
the stale claims marked `[stale]`. `none`, `narrow`, and `conflict` are two or
three lines. The first line is always `Recall: ACTION`.

## Reading

`show KEY` prints the live claim in every scope where the key is live:
`{"key","memories":[…]}`. A key that is not live anywhere is `no_memory`. The
fix is `log` when the key has history, and `remember` when it has none.

`log KEY` prints every generation of that key, in every scope, newest first,
capped at 50. `--scope` limits it to one scope. A tombstone has
`"forgotten": true` and a null claim. `log` is how a person audits why a claim
changed. A key with no generations is `no_memory`.

`relate-semantic` with no verb prints the scopes in the store, in name order,
with the number of live keys and generations in each. A missing store prints
`{"scopes":[]}`.

```json
{ "scopes": [{ "name": "inv-1042", "live": 0, "generations": 2 }, { "name": "org", "live": 1, "generations": 2 }] }
```

## Pruning

```bash
relate-semantic prune --scope inv-1042
```

`prune` deletes every generation in one scope, live and superseded, the scope
itself, and every subject no other memory uses. Other scopes are untouched. It
prints `{"pruned","keys","generations"}`. Pruning a scope with nothing in it
exits 0 with zero counts. A key written to a pruned scope starts again at
generation 1.

Pruning is the only delete. `forget` hides a claim. `prune` removes a scope's
history. The skill prunes a case scope when the relate-flow execution it
belongs to is pruned.

## CLI

```text
relate-semantic [-d PATH]
relate-semantic recall [--about SUBJ]... [--key KEY]... [--scope SCOPE]... [--fresh DURATION] [--limit N] [--output json|text] [-d PATH]
relate-semantic remember KEY --claim TEXT --about SUBJ... --scope SCOPE --because TEXT [--agent NAME] [-d PATH]
relate-semantic correct KEY --scope SCOPE --claim TEXT --because TEXT [--about SUBJ]... [--agent NAME] [-d PATH]
relate-semantic forget KEY --scope SCOPE --because TEXT [--agent NAME] [-d PATH]
relate-semantic show KEY [-d PATH]
relate-semantic log KEY [--scope SCOPE] [--json] [-d PATH]
relate-semantic prune --scope SCOPE [-d PATH]
relate-semantic skill install|remove|list|check [--agent NAME] [--force]
```

The first argument is the verb. Keys are positional after the verb, so no key
name is reserved.

`-d` / `--database` selects the store and may come before the verb or after
it. `RELATE_SEMANTIC_DATABASE` is the same value. The flag wins. The default is
`./.relate-semantic`.

`--agent` names the writer. `RELATE_SEMANTIC_AGENT` is the same value. The flag
wins. `remember`, `correct`, and `forget` require it. `recall` does not take
it. A recall returns every agent's claims.

`RELATE_SEMANTIC_NOW` is an RFC3339 clock for tests. The CLI reads it. The
library does not.

Every verb prints one compact JSON object on one line, except `recall
--output text`, and `log` on a terminal, which prints a table with columns
`GENERATION`, `SCOPE`, `AGENT`, `AT`, and `CLAIM`. `--json` forces JSON for
`log`. Errors are JSON on stdout on every path, including usage errors.
`--help` and `--version` print plain text and exit 0.

## Exit codes and errors

Stdout is one JSON object on every path. The exit code is how a harness
branches. The `fix` field is what the agent does.

| Exit | Meaning |
| --- | --- |
| 0 | An answer or a successful write, including `none`, `narrow`, `conflict`, and `stale`. |
| 2 | Usage. Unknown verb, missing flag, `recall` with no `--about` or `--key`, a bad duration, `--limit` outside 1 to 20, a write with no agent. |
| 4 | Refusal. The write breaks a rule. `fix` is the command to run instead. |
| 5 | Conflict. Two writes to the same key and scope raced, or the store is busy. |
| 1 | The store could not be opened or written, or a skill directory could not be written. |

```json
{
  "error": {
    "code": "key_taken",
    "message": "widget-price is already live in org: \"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.\"",
    "fix": "relate-semantic correct widget-price --scope org --claim TEXT --because TEXT",
    "retryable": false
  }
}
```

Every error has `code`, `message`, `fix`, and `retryable`. `retryable` is true
when the same command should be sent again.

`fix` is the whole corrected command, filled in from the call that was
refused, so an agent can run it after replacing the placeholders `TEXT`,
`SUBJ`, `SCOPE`, `KEY`, and `DURATION`. A refusal with several problems lists
every one in `errors`, each with `code`, `message`, `fix`, and `field`, the
flag at fault. The top-level `fix` repairs every slug at once. A usage error's
`fix` is the synopsis of the verb that was called.

| Code | Exit | When |
| --- | --- | --- |
| `usage` | 2 | A required flag is missing, or a value cannot be parsed. |
| `store` | 1 | The store could not be opened, read, or written. |
| `store_busy` | 5 | Another process held the store for more than two seconds. `retryable` is true. |
| `bad_slug` | 4 | A key, scope, subject, or agent is not a slug. |
| `empty_field` | 4 | The claim or the source is blank, or `remember` has no subject. |
| `too_long` | 4 | The claim or the source is over 600 characters. |
| `duplicate_value` | 4 | A subject, key, or scope is repeated. |
| `key_taken` | 4 | `remember` on a key already live in that scope with a different claim, source, or subjects. |
| `no_memory` | 4 | `correct` on a key that is not live in the scope, or `show` or `log` with nothing to show. |
| `foreign_store` | 4 | The store holds a label that relate-semantic and relate-flow do not write. |
| `conflict` | 5 | Another write to the same key and scope committed first. `retryable` is true. |

The `skill` verb adds `unknown_agent`, `no_agent`, `foreign_skill`, and
`skill_drift` (exit 4) and `skill_io` (exit 1), as in relate-flow.

## What is stored

The library writes a labeled property graph in the Relate database.

| What | Holds |
| --- | --- |
| `Memory` | `slot`, `key`, `scope`, `generation`, `claim`, `because`, `about`, `agent`, `at`, `forgotten`, `live`. One node per generation. |
| `Subject` | `name`. One node per subject slug. |
| `Scope` | `name`. One node per scope. |
| `ABOUT` | `Memory` to each of its `Subject`s. |
| `IN` | `Memory` to its `Scope`. |
| `SUPERSEDES` | A generation to the generation it replaced. |

`slot` is `scope/key/generation`, and it is unique. Two writers that both read
generation N and both create N+1 collide on it, and one of them gets
`conflict`. `Scope.name` and `Subject.name` are unique as well.

`live` is true on at most one generation per key and scope. It is set false on
the old generation in the same transaction that writes the new one, so a
recall never sees two live claims for one key in one scope. A tombstone is not
live. Apart from `live`, generations are immutable. `about` repeats the
subject names on the node so one query loads everything a command needs.

Relate Semantic writes `Memory`, `Scope`, and `Subject`. A relate-flow graph
(`Plan`, `Step`, and `Attempt`) may share the directory. A node with any
other label is refused with `foreign_store`.

## With relate-flow

Relate Flow says which step to do. Relate Semantic says what is already known
about it. Point both at the same directory with `-d`. Each tool reads and
writes only its own labels, and one process holds the directory at a time.
The skill gives the agent one loop:

1. `relate-flow CASE next --agent NAME` returns a `run`.
2. Name the subjects the step is about, from its title, `doneWhen`, and
   `inputs`.
3. `relate-semantic recall --about … --scope CASE --scope org`.
4. Do the step, using any claims that came back.
5. If a fact was learned that a later case would need, `remember` it in
   `org`. If a recalled claim was wrong, `correct` it. A fact only this case
   needs goes in the case scope.
6. `relate-flow CASE report STEP …`, then `next` again.

The flow note is the evidence for this step. The memory claim is the fact for
the next case. The same sentence is rarely both.

## Worked example

Two Northwind invoices arrive a week apart. The agent `invoices` drives each
one with the relate-flow `invoice` template.

```bash
export RELATE_SEMANTIC_AGENT=invoices
```

### The first invoice learns the agreement

`relate-flow inv-1042 next` leases `verify-discounts`. The agent recalls
before it reads the agreement.

```bash
relate-semantic recall --about northwind --about discounts --scope inv-1042 --scope org
```

```json
{ "action": "none", "about": ["discounts", "northwind"], "scopes": ["inv-1042", "org"] }
```

The agent reads NW-2024, finds the 10% rule, and computes the $1,000
shortfall. The rule outlives this invoice, so it goes in `org`. The shortfall
is this invoice's evidence, so it goes in the flow note.

```bash
relate-semantic remember widget-price \
  --claim "Under NW-2024, widgets are \$100, with 10% off a widget line over \$5000." \
  --about northwind --about nw-2024 --about discounts \
  --scope org --because "Read from the agreement while checking INV-1042."

relate-flow inv-1042 report verify-discounts --outcome succeeded --result incorrect \
  --note "NW-2024 gives 10% off this line. Agreed total \$9,000, billed \$10,000, shortfall \$1,000."
```

The person reviewing the escalation approves $9,000, and says Northwind's
billing team is to be told about the shortfall on every invoice. That is a
standing practice, so the agent stores it.

```bash
relate-semantic remember shortfall-notice \
  --claim "Tell Northwind billing about every discount shortfall when the invoice is paid." \
  --about northwind --about discounts \
  --scope org --because "Approved by the reviewer on INV-1042."
```

### The second invoice uses it

`relate-flow inv-1077 next` leases `verify-discounts`. The same recall now
returns both claims.

```json
{
  "action": "use",
  "memories": [
    { "key": "shortfall-notice", "scope": "org", "claim": "Tell Northwind billing about every discount shortfall when the invoice is paid.", "…": "…" },
    { "key": "widget-price", "scope": "org", "claim": "Under NW-2024, widgets are $100, with 10% off a widget line over $5000.", "…": "…" }
  ]
}
```

The agent applies the rule without re-reading the agreement, and puts the
notice in the `pay` step.

### A case overrides the shared fact

INV-1077 carries a one-off price agreed by phone. The agent stores it in the
case scope only.

```bash
relate-semantic remember widget-price \
  --claim "For INV-1077 only, widgets are \$95, agreed by phone with Northwind on 2026-11-15." \
  --about northwind --about nw-2024 --about discounts \
  --scope inv-1077 --because "Phone note attached to INV-1077."
```

`recall --scope inv-1077 --scope org` now returns the `inv-1077` claim for
`widget-price`. A recall from another case still returns the `org` claim. A
recall with no `--scope` returns `conflict` on `widget-price`, because the
agent has not said which scope should win.

When INV-1077 is finished, the case scope goes with it.

```bash
relate-flow invoice prune
relate-semantic prune --scope inv-1077
```

### The agreement changes

NW-2025 replaces NW-2024. The next agent to read the new agreement corrects
the shared claim.

```bash
relate-semantic correct widget-price --scope org \
  --claim "Under NW-2025, widgets are \$110, with 10% off a widget line over \$5000." \
  --about northwind --about nw-2025 --about discounts \
  --because "NW-2025 replaced NW-2024 on 2026-11-01."
```

`log widget-price` shows both generations. A recall with `--about nw-2024`
no longer matches, because the live generation is about `nw-2025`.

## First version

The first version is the library, `remember`, `correct`, `forget`, `recall`,
`show`, `log`, `prune`, the store rules above, and the skill.

It does not add similarity search, automatic extraction, expiry, cross-store
recall, access control between agents, or a Cypher surface for agents.

## Skill

`relate-semantic skill install` writes an agent skill that ships inside the
binary. The skill teaches the agent when to recall, what is worth
remembering, which scope a fact belongs in, and the loop with relate-flow
above. It says, first: recall before you act, and remember a fact that
outlives the case.

The skill is a verb on the CLI. It never opens a store. It still prints one
JSON object and uses the same exit codes as the relate-flow `skill` verb.
