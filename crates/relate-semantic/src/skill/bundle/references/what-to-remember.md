# What to remember

A memory is for the next agent. It is worth writing when that agent would
otherwise have to learn the same thing again, from a document, from a person,
or from a failure. Before you `remember`, answer five questions.

## 1. Would a later case need it?

Remember:

- A fact you read from a source that later cases will consult again. The
  NW-2024 price.
- A ruling a person made that later cases are judged by. "A transliterated
  name is approved when the date of birth and document number match."
- What worked, or did not, when something broke. "Drain the checkout pool,
  then roll back."
- A known hit or exception about an entity. "Orion Trading LLC is on the EU
  sanctions list."

Do not remember:

- The case's own numbers: this invoice's total, this applicant's name. They
  belong in the flow note.
- What you did step by step. That is the flow log.
- Something any agent can see in the input in front of it.
- A guess. If you are not sure, say so in the flow note and remember nothing.

## 2. Who is it true for?

That picks the scope.

| Scope | Holds | Example |
| --- | --- | --- |
| `org` | Facts for every case, until corrected or forgotten. | Agreement prices, rulings, runbooks. |
| `case-…` | Facts for one case, pruned when the case closes. | A phone quote for this invoice only, a note for the retry. |

A ruling made on one case that applies to later cases goes in `org`. A fact
that overrides `org` for one case goes in the case scope under the same key,
and the case's recalls list the case scope first.

## 3. What will a later agent recall it by?

Those are the subjects. Recall matches every `--about` it is given, so pick
subjects a later agent will name without knowing your case:

- The party: `northwind`, `orion-trading`.
- The document or system: `nw-2024`, `checkout`, `dashboards`.
- The topic: `discounts`, `runbook`, `name-mismatch`, `sanctions`.

Two to four subjects is usual. Do not use the case number as a subject for an
`org` fact; nobody will recall by it.

## 4. Where did it come from?

That is `--because`. Name the document and section, the case and review, the
incident and postmortem, or the person and date. A later agent uses it to
check a stale claim, and a person uses it to audit a decision.

## 5. Is it one fact?

`--claim` is at most 600 characters. Write the fact so the next agent can act
on it without opening the source:

- Good: `Under NW-2024, widgets are $100, with 10% off a widget line over $5000.`
- Too little: `See NW-2024 section 3.`
- Too much: the whole agreement pasted in.

The key names the fact, so it survives a correction: `widget-price`, not
`widget-price-100`.

## When a remembered fact changes

- The fact changed: `correct` it, and say why in `--because`. The old claim
  stays in `log`.
- The subjects changed: `correct --about …` replaces them all. A recall by a
  dropped subject no longer finds it.
- You checked it and it still holds: `correct` with the same claim and a new
  `--because`. That resets its age for `--fresh`.
- It no longer holds and nothing replaces it: `forget` it.
- The case is closed: `prune --scope CASE`.
