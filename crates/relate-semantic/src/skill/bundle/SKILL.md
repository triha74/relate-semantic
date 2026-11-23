---
name: relate-semantic
description: Relate Semantic keeps the facts agents chose to remember, each with its source. Use this skill to recall what is already known before a step, and to remember a fact a later case would otherwise have to work out again.
version: {{VERSION}}
---

# Relate Semantic

Relate Semantic is not a model. It does not read your conversation, extract
facts, summarize, or decide which of two claims is true. You write the
sentence the next agent will read, and you say where it came from. It decides
whether the claim may be stored, which claim is live, and what a recall
returns. The binary is `relate-semantic`. One invocation opens the store, runs
one transaction, prints one JSON object, and exits.

Recall before you act. Remember a fact that outlives the case.

## The loop

With relate-flow, each step gets the same loop:

1. `relate-flow CASE next --agent NAME` returns a `run`.
2. Name the subjects the step is about, from its title, `doneWhen`, and
   `inputs`: the customer, the agreement, the system, the topic.
3. `relate-semantic recall --about SUBJ --about SUBJ --scope CASE --scope org`.
4. Do the step, using the claims that came back as written.
5. If you learned a fact a later case would need, `remember` it. If a
   recalled claim was wrong, `correct` it. If it no longer holds, `forget` it.
6. `relate-flow CASE report STEP …`, then `next` again.

Without relate-flow the loop is the same: recall at the start of a piece of
work, remember at the end of it.

The flow note is the evidence for this step. The memory claim is the fact for
the next case. The same sentence is rarely both.

## Setup

```bash
export RELATE_SEMANTIC_AGENT=invoices
```

The default store is `./.relate-semantic`. `-d PATH` and
`RELATE_SEMANTIC_DATABASE` select another. `relate-semantic` with no verb lists the
scopes in the store. `remember`, `correct`, and `forget` need an agent:
`--agent NAME` or `RELATE_SEMANTIC_AGENT`. The flag wins.

Keys, scopes, subjects, and agents are slugs: a lowercase letter, then
lowercase letters, digits, and hyphens. `widget-price`, `case-inv-1042`,
`nw-2024`.

## Recall

```bash
relate-semantic recall --about northwind --about discounts --scope case-inv-1042 --scope org
```

- `--about` repeats. A memory matches when it carries every subject (AND).
  Fewer subjects find more. There is no OR; run two recalls.
- `--key` repeats, when you know the key.
- `--scope` repeats, in the order that should win. Put the case scope first
  and `org` last. Leave `--scope` out only when you want to see every scope.
- `--fresh 30d` flags a winning claim older than 30 days.
- `--limit N` is the most keys a `use` returns, 1 to 20. The default is 5.
- `--output text` prints the answer as a block for your prompt. The first line
  is `Recall: ACTION`.

Switch on `action`. Every action exits 0. They are answers.

| `action` | What you do |
| --- | --- |
| `use` | Use the claims as written. Do not re-derive them. Cite `because` if a person will review the work. |
| `none` | Nothing is known. Do the work. Then remember a fact a later case would need. |
| `conflict` | One key is live in two scopes and you did not say which wins. Run `fix` with the scopes in the order that should win. Do not pick a claim yourself. |
| `narrow` | Too many keys matched. `keys` lists them with their subjects. Recall again with `--key`, or add an `--about`. |
| `stale` | A claim is older than `--fresh`. The claims are still there. Check the source in `because`. If it changed, `correct` the claim. If it still holds, `correct` with the same claim and a `--because` naming your check. |

Recall again after a `conflict` or a `narrow`. Do not act on a fact two scopes
disagree about.

## Remember

```bash
relate-semantic remember widget-price \
  --claim 'Under NW-2024, widgets are $100, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2024 --about discounts --scope org \
  --because 'Read from NW-2024 section 3 while checking INV-1042.'
```

Read [what to remember](references/what-to-remember.md) before you write. The
short form:

1. Remember a fact a later case would otherwise work out again from a
   document, a person, or a failure. Do not remember the case's own numbers.
2. `org` holds facts for everyone. A case scope such as `case-inv-1042` holds
   facts for one case. A ruling that applies to later cases goes in `org`,
   even when one case produced it.
3. `--about` is what a later agent will recall by. Use the customer, the
   agreement, the system, and the topic, not the case number.
4. `--claim` is one fact in a sentence or two, at most 600 characters. Write
   it so the next agent can use it without the source.
5. `--because` says where it came from: the document and section, the case,
   the review, the person. At most 600 characters.
6. The key names the fact, not the claim: `widget-price`, not
   `widget-price-is-100`.

Repeating the same `remember` returns the stored generation and writes
nothing, so a retry is safe. A different claim on a live key is refused with
`key_taken`, and `fix` is the `correct` command.

## Correct, forget, prune

```bash
relate-semantic correct widget-price --scope org \
  --claim 'Under NW-2025, widgets are $110, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2025 --about discounts \
  --because 'NW-2025 replaced NW-2024 on 2026-11-01.'
relate-semantic forget rollback-blocked --scope case-inc-313 --because 'The retried rollback stuck.'
relate-semantic prune --scope case-inv-1042
```

- `correct` writes the next generation. `--about` replaces every subject.
  Leave it out to keep them. Say in `--because` why the claim changed.
- `forget` stops a key being live. The history stays. To bring it back,
  `remember` it. Forgetting twice is safe.
- `prune` deletes a whole scope and its history. Prune a case scope when the
  case is closed. Never prune `org`.

## Reading

- `relate-semantic show KEY` prints the live claim in every scope.
- `relate-semantic log KEY [--scope SCOPE] [--json]` prints every generation,
  newest first, with who wrote it and why. Read it before you correct a claim
  someone else wrote.

## Errors

Stdout is one JSON object on every path, including a mistake.

| Exit | Meaning |
| --- | --- |
| 0 | An answer or a successful write, including `none`, `narrow`, `conflict`, and `stale`. |
| 2 | Usage. A missing flag, no `--about` or `--key` on `recall`, a bad `--fresh` or `--limit`, no agent on a write. |
| 4 | Refusal. A bad slug, blank or long text, `key_taken`, `no_memory`. |
| 5 | The store is busy, or another write to the same key won. Run the same command again. |
| 1 | The store or a skill directory could not be read or written. |

`fix` is the command to run instead, filled in from your call, with every
slug corrected. Replace `TEXT`, `SUBJ`, `SCOPE`, `KEY`, or `DURATION` and run
it. When `errors` is present, it lists every problem with the flag at fault,
and the one `fix` repairs every slug. The codes are in
[errors](references/errors.md).

The [invoice](references/invoice.md) walkthrough and the incident and
onboarding patterns in [scenarios](references/scenarios.md) show what to
recall and remember at each step of the relate-flow examples.

Do not write Cypher against the store. Cypher is not the agent API.
