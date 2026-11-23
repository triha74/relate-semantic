# Errors

Every error is one JSON object on stdout:

```json
{"error":{"code":"key_taken","fix":"relate-semantic correct widget-price --scope org --claim TEXT --because TEXT","message":"widget-price is already live in org: \"Under NW-2024, widgets are $100, with 10% off a widget line over $5000.\"","retryable":false}}
```

Do what `fix` says. It is the whole command, filled in from your call. Every
slug in it is already corrected and every repeated value is dropped. Replace
the placeholders `TEXT`, `SUBJ`, `SCOPE`, `KEY`, and `DURATION` and run it. A
few fixes are a sentence, because no command repairs them.

When `errors` is present, it lists every problem, each with `code`, `message`,
`fix`, and `field`, the flag at fault. The top-level `fix` repairs them all at
once, so one retry is enough:

```json
{"error":{"code":"bad_slug","errors":[{"code":"bad_slug","field":"about","fix":"pass --about passport-scan","message":"--about 'Passport Scan' is not a slug"}],"fix":"relate-semantic remember app-4-document --claim TEXT --about passport-scan --scope case-app-4 --because TEXT","message":"--about 'Passport Scan' is not a slug","retryable":false}}
```

`retryable` is true for `conflict` and `store_busy`. Run the same command
again. Writes are idempotent, so the rerun is safe.

| Code | Exit | When | Fix |
| --- | --- | --- | --- |
| `usage` | 2 | A flag is missing, unknown, or unreadable. `recall` has no `--about` or `--key`. `--limit` is outside 1 to 20. `--fresh` is not a duration. A write has no agent. | The verb's synopsis, or the corrected command. |
| `store` | 1 | The store could not be opened, read, or written. | Check the path given to `-d`. |
| `store_busy` | 5 | Another process held the store too long. | Run the same command again. |
| `conflict` | 5 | Another write to the same key committed first. | Run the same command again. |
| `foreign_store` | 4 | The directory holds a label that relate-semantic and relate-flow do not write. | Use another `-d`. |
| `bad_slug` | 4 | A key, scope, subject, or agent is not a slug. | The command with the slug corrected. |
| `empty_field` | 4 | `--claim` or `--because` is blank, or `remember` has no `--about`. | The command with `TEXT` or `SUBJ` to fill in. |
| `too_long` | 4 | `--claim` or `--because` is over 600 characters. | Write only the fact. Put the source in `--because`. |
| `duplicate_value` | 4 | The same `--about`, `--key`, or `--scope` was passed twice. | The command with each value once. |
| `key_taken` | 4 | `remember` on a key already live in the scope with something different. `message` quotes the live claim. | The `correct` command. Read the live claim first. |
| `no_memory` | 4 | `correct` on a key that is not live, or `show` or `log` with nothing to show. | The `remember` command, or `log` for a forgotten key. |
| `unknown_agent` | 4 | `skill --agent` names an unknown agent. | Pick from the list in `fix`. |
| `no_agent` | 4 | `skill` found no agent to install for. | Pass `--agent NAME`. |
| `foreign_skill` | 4 | The skill directory was not installed by relate-semantic. | On `install`, `relate-semantic skill install --agent NAME --force`. On `remove`, leave the directory in place. |
| `skill_drift` | 4 | An installed skill differs from this binary. | `relate-semantic skill install`. |
| `skill_io` | 1 | A skill directory could not be written. | Check that the skills directory is writable. |

The recall actions are not errors. `conflict`, `narrow`, `stale`, `use`, and
`none` all exit 0, and only `conflict` and `narrow` carry a `fix`, which is
the recall to run next.
