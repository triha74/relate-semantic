# Invoice

What the `invoices` agent recalls and remembers while relate-flow's invoice plan pays INV-1042, or hands it to a person. Each script has the same name as a script in relate-flow's `examples/invoice`, and a comment above each block names the flow step it goes with. `seed.sh` picks the store and writes the org-wide review rule every case starts from. `seed_price` writes the NW-2024 price the way an earlier case would have.

The case scope is `case-inv-1042`. Recalls name it first, so a case note wins over an org fact with the same key.

| Script | What it shows | Ends with |
| --- | --- | --- |
| `run-discount-missing.sh` | `verify-discounts` recalls nothing and remembers the NW-2024 price. After `human-review` approves $9,000, the agent remembers how a shortfall is handled. A week later, INV-1077 recalls both. | `use` |
| `run-discount-rejected.sh` | The rejection is about INV-1042 only, so it goes in the case scope. `close-rejected` prunes the scope, and the note is gone. | `none` |
| `run-discount-correct.sh` | The price is already remembered, so `verify-pricing` uses it without opening the agreement. Remembering the same fact again returns the stored generation. | `use` |
| `run-vendor-rejected.sh` | `verify-vendor` remembers that Northwind is not approved. The person approves payment anyway, and the agent corrects the memory instead of writing a new key. `log` shows both generations. | `use` |
| `run-missing-agreement.sh` | NW-2024 is missing, so a phone quote goes in the case scope. A second `remember` in that scope is refused with `key_taken`. When NW-2024 arrives the price goes in org, and a recall that names no scope returns `conflict`. | `conflict` |

```bash
cargo build --release -p relate-semantic
bash examples/run-scenario.sh invoice
bash examples/run-scenario.sh -v invoice run-missing-agreement
```

Each script prints only the CLI's JSON. The last recall in `run-missing-agreement.sh` is:

```json
{"action":"conflict","fix":"relate-semantic recall --key widget-price --scope case-inv-1042 --scope org","key":"widget-price","scopes":["case-inv-1042","org"]}
```

`docs/scenarios/invoice.md` walks each script next to the relate-flow step it goes with.
