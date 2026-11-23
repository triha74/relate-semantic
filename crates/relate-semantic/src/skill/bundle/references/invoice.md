# Invoice

The pattern for a fact one case learns and every later case uses, and for a
case override that must not leak into other cases.

The relate-flow plan pays INV-1042, or hands it to a person. The agent is
`invoices`. The case scope is `case-inv-1042`. Every recall in the case lists
it first: `--scope case-inv-1042 --scope org`.

## verify-discounts: recall, then read

```bash
relate-semantic recall --about northwind --about discounts --scope case-inv-1042 --scope org
```

```json
{"about":["northwind","discounts"],"action":"none","keys":[],"scopes":["case-inv-1042","org"]}
```

Nothing is known, so the agent reads NW-2024. The price applies to every
Northwind invoice, so it goes in `org`:

```bash
relate-semantic remember widget-price \
  --claim 'Under NW-2024, widgets are $100, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2024 --about discounts --scope org \
  --because 'Read from NW-2024 section 3 while checking INV-1042.'
```

The $1,000 shortfall is this invoice's evidence. It goes in the flow note, not
in memory.

## human-review: the ruling, not the amount

The person approves $9,000. The rule behind it outlives the case:

```bash
relate-semantic remember shortfall-handling \
  --claim 'When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.' \
  --about northwind --about discounts --about shortfall --scope org \
  --because 'REV-19 approved $9,000 for INV-1042 on 2026-11-10.'
```

## The next invoice

INV-1077 recalls with its own case scope first and gets `use` with both
claims. The agent applies them without opening the agreement.

## A case override

When NW-2024 is missing and finance quotes a price by phone for this invoice
only, the quote goes in the case scope under the same key:

```bash
relate-semantic remember widget-price \
  --claim 'For INV-1042 only, finance quoted $100 per widget by phone while NW-2024 was missing.' \
  --about northwind --about nw-2024 --scope case-inv-1042 \
  --because 'Phone call with finance on 2026-11-10. NW-2024 was not on file.'
```

Recalls for INV-1042 list the case scope first and get the quote. Recalls for
any other invoice leave it out and get the `org` price. A recall with no
`--scope` returns `conflict`, because you have not said which scope wins:

```json
{"action":"conflict","fix":"relate-semantic recall --key widget-price --scope case-inv-1042 --scope org","key":"widget-price","scopes":["case-inv-1042","org"]}
```

## When the agreement changes

```bash
relate-semantic correct widget-price --scope org \
  --claim 'Under NW-2025, widgets are $110, with 10% off a widget line over $5000.' \
  --about northwind --about nw-2025 --about discounts \
  --because 'NW-2025 replaced NW-2024 on 2026-11-01.'
```

Not a new key. `log widget-price` keeps the NW-2024 generation and its
source. A recall by `nw-2024` no longer matches.

## close-rejected or pay

The case is finished. Nothing in its scope helps a later case:

```bash
relate-semantic prune --scope case-inv-1042
```
