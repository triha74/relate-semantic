# Onboarding

What the `kyc` and `compliance` agents recall and remember while relate-flow's onboarding plan opens an account, or declines the application. Each script has the same name as a script in relate-flow's `examples/onboarding`. `seed.sh` picks the store and writes the accepted identity documents every application starts from.

Each application has its own scope, such as `case-app-4`. Rulings that apply to later applicants go in `org`.

| Script | What it shows | Ends with |
| --- | --- | --- |
| `run-clear.sh` | `risk-score` remembers the low-risk threshold the analyst used. The next applicant recalls it. | `use` |
| `run-mismatch-approved.sh` | `compliance-review` approves a transliterated name. The ruling goes in `org` under the `compliance` agent, and the next mismatch recalls it. | `use` |
| `run-hit-declined.sh` | A sanctions hit is about the entity, not the application, so it goes in `org`. A later application with the same owner recalls it at `sanctions-screen`. | `use` |
| `run-blocked.sh` | The document can't be read. Pasting the OCR dump as a claim is refused with `too_long`, and a subject with a space is refused with `bad_slug`. The agent follows the `fix` and writes the fact. | `use` |
| `run-reopen-donewhen.sh` | Compliance changes what counts as proof of address. `correct --about` replaces the subjects, and a recall by the old subject returns nothing. | `none` |

```bash
cargo build --release -p relate-semantic
bash examples/run-scenario.sh onboarding
bash examples/run-scenario.sh -v onboarding run-blocked
```

The `bad_slug` refusal in `run-blocked.sh` carries the corrected command:

```json
{"error":{"code":"bad_slug","errors":[{"code":"bad_slug","field":"about","fix":"pass --about passport-scan","message":"--about 'Passport Scan' is not a slug"}],"fix":"relate-semantic remember app-4-document --claim TEXT --about passport-scan --scope case-app-4 --because TEXT","message":"--about 'Passport Scan' is not a slug","retryable":false}}
```

`docs/scenarios/onboarding.md` walks each script next to the relate-flow step it goes with.
