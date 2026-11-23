# Incident

What the `oncall` agent recalls and remembers while relate-flow's incident plan restores a service, or escalates it. Each script has the same name as a script in relate-flow's `examples/incident`. `seed.sh` picks the store and writes the severity guide every incident starts from. `seed_runbook` writes the checkout runbook the way an earlier incident would have.

Each incident has its own scope, such as `case-inc-311`. Runbooks and known causes go in `org`.

| Script | What it shows | Ends with |
| --- | --- | --- |
| `run-sev1-resolved.sh` | `triage` uses the severity guide. `mitigate` finds no runbook. The fix works, and `postmortem` remembers it with the incident as the source. Ten days later the next outage recalls it. | `use` |
| `run-sev2-unresolved.sh` | The runbook comes back, the rollback does not fix search, and the vendor is paged. The agent corrects the runbook and adds `search` to its subjects. | `use` |
| `run-sev3.sh` | The dashboard runbook is 45 days old. `recall --fresh 30d` returns `stale` and names the claim to check. | `stale` |
| `run-retry.sh` | A note from the failed first attempt is wrong once the retry works, so it is forgotten. Forgetting it again returns `"forgotten": false`. | `none` |
| `run-replan.sh` | The plan is revised to bring the vendor in. A recall for every checkout memory matches six keys, more than the limit of five, and returns `narrow` with the keys to choose from. | `narrow` |

```bash
cargo build --release -p relate-semantic
bash examples/run-scenario.sh incident
bash examples/run-scenario.sh -text incident run-sev3
```

The last recall in `run-sev3.sh` is:

```json
{"action":"stale","fresh":"30d","memories":[{"about":["dashboards","runbook"],"agent":"oncall","at":"2026-11-10T18:00:00Z","because":"INC-288: refreshing the cache fixed the revenue chart.","claim":"A stale chart is fixed by refreshing the chart cache from the dashboards admin page.","generation":1,"key":"chart-refresh","scope":"org"}],"stale":["chart-refresh"]}
```

`docs/scenarios/incident.md` walks each script next to the relate-flow step it goes with.
