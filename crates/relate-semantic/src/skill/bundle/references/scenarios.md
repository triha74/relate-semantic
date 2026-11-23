# Scenarios

Patterns from the relate-flow incident and onboarding examples. They show
what to recall at each step and where each fact goes. They are not scripts to
copy.

## Incident

The agent is `oncall`. Each incident has a case scope, `case-inc-311`.
Runbooks and known causes go in `org`.

| Step | Recall or write |
| --- | --- |
| `triage` | `recall --about severity --scope case-inc-311 --scope org` |
| `mitigate` | `recall --about checkout --about runbook --scope case-inc-311 --scope org`. Add `--fresh 30d` when an old runbook could be wrong. |
| `mitigate` failed | `remember` a note for the retry in `case-inc-311`. Forget it when the retry works. |
| `postmortem` | `remember` what worked in `org`, with the incident and the postmortem in `--because`. Or `correct` the runbook that came back and did not work. |

A runbook that comes back `stale` is still returned. Check it. If the fix
changed, `correct` it. If it still works, `correct` it with the same claim and
a `--because` that names the incident where you checked it. When the plan is
revised and you widen a recall to every memory about a system, expect
`narrow`, and pick keys from its list.

```json
{"action":"narrow","fix":"relate-semantic recall --about checkout --key checkout-db","keys":[{"about":["checkout","database"],"key":"checkout-db","scope":"org"},{"about":["checkout","flags"],"key":"checkout-flags","scope":"org"}],"limit":1,"matched":2}
```

## Onboarding

The agents are `kyc` and `compliance`. Each application has a case scope,
`case-app-4`. Rulings that later applicants are judged by go in `org`, written
by the agent that made them.

| Step | Recall or write |
| --- | --- |
| `verify-identity` | `recall --about identity`, and `--about name-mismatch` on a mismatch. |
| `sanctions-screen` | `recall --about ENTITY` for each owner, such as `orion-trading`. |
| `risk-score` | `recall --about risk --about threshold`. |
| `compliance-review` | `remember` the ruling in `org` with `--agent compliance`. A hit is about the entity, so the entity is a subject. |
| `collect-docs` reopened | `correct --about …` when what counts as a document changes, so the old subject stops matching. |

When a document can't be read, do not paste it into `--claim`. A claim over
600 characters is refused with `too_long`. Write one sentence about what is
wrong and what to ask for, in the case scope.

```json
{"error":{"code":"too_long","errors":[{"code":"too_long","field":"claim","fix":"write only the fact in --claim, and put the source in --because","message":"--claim is 815 characters and the limit is 600"}],"fix":"relate-semantic remember app-4-document --claim TEXT --about passport-scan --scope case-app-4 --because TEXT","message":"--claim is 815 characters and the limit is 600","retryable":false}}
```
