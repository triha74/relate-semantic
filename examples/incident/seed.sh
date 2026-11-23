#!/usr/bin/env bash
# Every incident starts from the severity guide. seed_runbook writes the
# checkout runbook the way an earlier incident would have.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../bin.sh"
example_store
export RELATE_SEMANTIC_AGENT=oncall

RUNBOOK='When checkout is down after a deploy, drain the checkout pool, then roll back the deploy.'
RUNBOOK_SRC='INC-311: draining and rolling back restored checkout in 12 minutes. PM-14.'

mem remember severity-guide \
  --claim 'Sev1 means checkout or login is down for every customer. Sev2 means a feature is degraded. Anything else is sev3.' \
  --about severity --about triage --scope org \
  --because 'Incident policy IP-2, 2026-03-01.' >/dev/null

seed_runbook() {
  mem remember checkout-runbook --claim "$RUNBOOK" \
    --about checkout --about runbook --scope org --because "$RUNBOOK_SRC"
}
