# From a task to a remembered fact

You hand an agent an invoice and one sentence: pay it, or hand it to a person. relate-flow drives the steps. relate-semantic is what the agent checks before each step and writes to when it learns something the next case will need.

The agent does the reading, the checking, and the payment. relate-semantic keeps the facts the agent chose to keep, and says plainly when there are none, when they disagree, or when they are old.

The output below is from one run of `examples/invoice/run-discount-missing.sh` and `run-missing-agreement.sh`. Times come from `RELATE_SEMANTIC_NOW`.

## The skill arrives first

```bash
relate-semantic skill install
```

Install writes `SKILL.md` and four references into each agent it detects. For Cursor that directory is `~/.cursor/skills/relate-semantic/`. The front matter tells the agent when to load it: before a step that needs facts it might already know, and after a step that taught it something a later case would need.

Before the agent writes anything, the skill sends it to [what to remember](../crates/relate-semantic/src/skill/bundle/references/what-to-remember.md). The test there is short.

1. Would a later case have to work this out again, from a document, a person, or a failure? Then remember it.
2. Is it true for everyone, or only for this case? That picks the scope.
3. What would a later agent recall it by? Those are the subjects.
4. Where did it come from? That is `--because`.
5. Is it one fact in a sentence or two? A document dump is not a claim.

## The task, before any command

> Pay this invoice, or hand it to a person. Northwind bills under agreement NW-2024. INV-1042 is 100 widgets and bills $10,000.

The agent is `invoices`. It sets that once:

```bash
export RELATE_SEMANTIC_AGENT=invoices
```

It names the case scope `case-inv-1042`, and uses `org` for facts that hold for every Northwind invoice. Every recall in this case lists the case scope first, so a note about INV-1042 wins over an org fact with the same key.

## verify-discounts: recall before reading

relate-flow hands out `verify-discounts`. Before opening NW-2024, the agent asks what is already known about Northwind discounts:

```bash
relate-semantic recall --about northwind --about discounts --scope case-inv-1042 --scope org
```

```json
{"about":["northwind","discounts"],"action":"none","keys":[],"scopes":["case-inv-1042","org"]}
```

`none` echoes the query, so the agent can see it asked the right thing. Nothing is known. It reads section 3 of NW-2024: widgets are $100, with 10% off a widget line over $5,000. The invoice has no discount, so the agreed total is $9,000 and the bill is $1,000 too high. It reports `incorrect` to relate-flow.

## Remember what the next case would need

The NW-2024 price will be needed by every Northwind invoice, so it goes in `org`. The subjects are the words a later agent would recall with: the customer, the agreement, and the topic.

```bash
relate-semantic remember widget-price \
  --claim 'Under NW-2024, widgets are $100, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2024 --about discounts --scope org \
  --because 'Read from NW-2024 section 3 while checking INV-1042.'
```

```json
{"at":"2026-11-10T18:00:00Z","generation":1,"key":"widget-price","scope":"org"}
```

The shortfall amount is about INV-1042 only. The agent leaves it in the relate-flow note, where `human-review` will read it, and does not remember it.

## human-review: remember the ruling, not the case

The person approves $9,000. The amount is about this invoice. The rule behind it is not: when a Northwind invoice leaves out the discount, pay the agreed total. A later agent should apply that without asking again.

```bash
relate-semantic remember shortfall-handling \
  --claim 'When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.' \
  --about northwind --about discounts --about shortfall --scope org \
  --because 'REV-19 approved $9,000 for INV-1042 on 2026-11-10.'
```

```json
{"at":"2026-11-10T18:00:00Z","generation":1,"key":"shortfall-handling","scope":"org"}
```

If the command times out and the agent runs it again, the answer is the same object and nothing new is written.

## The next case recalls instead of reading

A week later, INV-1077 arrives. `verify-discounts` recalls with the new case scope first:

```bash
relate-semantic recall --about northwind --about discounts --scope case-inv-1077 --scope org
```

```json
{"action":"use","memories":[{"about":["discounts","northwind","shortfall"],"agent":"invoices","at":"2026-11-10T18:00:00Z","because":"REV-19 approved $9,000 for INV-1042 on 2026-11-10.","claim":"When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.","generation":1,"key":"shortfall-handling","scope":"org"},{"about":["discounts","northwind","nw-2024"],"agent":"invoices","at":"2026-11-10T18:00:00Z","because":"Read from NW-2024 section 3 while checking INV-1042.","claim":"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.","generation":1,"key":"widget-price","scope":"org"}]}
```

`use` means: use these claims as written. The agent checks the invoice against the price and the rule without opening NW-2024. Each claim says where it came from, so a person reviewing the payment can follow it back.

With `--output text`, the same answer is a block the agent can paste into its own prompt. The first line is always `Recall: ACTION`.

```text
Recall: use 2 memories.
Use these claims as written. Do not re-derive them.

shortfall-handling (org, generation 1, 2026-11-10T18:00:00Z, by invoices)
When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.
Source: REV-19 approved $9,000 for INV-1042 on 2026-11-10.

widget-price (org, generation 1, 2026-11-10T18:00:00Z, by invoices)
Under NW-2024, widgets are $100, with 10% off a widget line over $5000.
Source: Read from NW-2024 section 3 while checking INV-1042.
```

## When the agreement is missing

In another run, NW-2024 is not on file when `verify-pricing` starts. Finance quotes $100 per widget by phone. That quote holds for INV-1042 only, so it goes in the case scope:

```bash
relate-semantic remember widget-price \
  --claim 'For INV-1042 only, finance quoted $100 per widget by phone while NW-2024 was missing.' \
  --about northwind --about nw-2024 --scope case-inv-1042 \
  --because 'Phone call with finance on 2026-11-10. NW-2024 was not on file.'
```

An hour later NW-2024 arrives, and the agent tries to write the agreement price under the same key and scope:

```json
{"error":{"code":"key_taken","fix":"relate-semantic correct widget-price --scope case-inv-1042 --claim TEXT --because TEXT --about discounts --about northwind --about nw-2024","message":"widget-price is already live in case-inv-1042: \"For INV-1042 only, finance quoted $100 per widget by phone while NW-2024 was mis…\"","retryable":false}}
```

Exit 4. `message` quotes what is already there, and `fix` is the `correct` command with the new subjects filled in. The agent sees the real mistake: the agreement price is not a case fact. It writes it in `org` instead, and leaves the phone note where it is.

Now `widget-price` is live in two scopes. A recall that does not say which scope wins gets an answer that says so:

```bash
relate-semantic recall --about northwind --about nw-2024
```

```json
{"action":"conflict","fix":"relate-semantic recall --key widget-price --scope case-inv-1042 --scope org","key":"widget-price","scopes":["case-inv-1042","org"]}
```

`fix` is the recall to run, with the scopes in name order. The agent puts the scope that should win first. For INV-1042 that is the case scope. For any other invoice it leaves the case scope out.

## Facts that change

When NW-2025 replaces NW-2024, the agent does not write a new key. It corrects the old one, and says why:

```bash
relate-semantic correct widget-price --scope org \
  --claim 'Under NW-2025, widgets are $110, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2025 --about discounts \
  --because 'NW-2025 replaced NW-2024 on 2026-11-01.'
```

```json
{"at":"2026-11-17T17:00:00Z","generation":2,"key":"widget-price","scope":"org"}
```

`--about` replaced the subjects, so a recall by `nw-2024` no longer finds the price. `relate-semantic log widget-price` still shows generation 1 with its source.

A claim nobody has checked in a while can be flagged at recall. `--fresh 30d` returns `stale` instead of `use` when a winning claim is older than 30 days, with the claims still attached:

```text
Recall: stale widget-price.
Older than 30d: widget-price. Check the source in each, then correct it, or recall without --fresh.

[stale] widget-price (org, generation 2, 2026-11-17T17:00:00Z, by invoices)
Under NW-2025, widgets are $110, with 10% off a widget line over $5000.
Source: NW-2025 replaced NW-2024 on 2026-11-01.
```

## Closing the case

When INV-1042 is closed, nothing in `case-inv-1042` matters to a later case. The agent prunes it:

```bash
relate-semantic prune --scope case-inv-1042
```

```json
{"generations":1,"keys":1,"pruned":"case-inv-1042"}
```

The org facts stay. The next Northwind invoice starts with the price, the rule, and nothing from INV-1042's phone call.
