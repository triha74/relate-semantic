#!/usr/bin/env bash
# expect: use
# The runbook comes back, the rollback does not fix search, and the vendor
# is paged. The agent corrects the runbook so the next responder pages the
# vendor sooner. The subjects change with it.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
seed_runbook >/dev/null
CASE=case-inc-312

# mitigate
mem recall --about checkout --about runbook --scope "$CASE" --scope org

# mitigate unresolved, vendor-escalation paged, postmortem filed
advance 3h
mem correct checkout-runbook --scope org \
  --claim 'When checkout is down after a deploy, drain the checkout pool, then roll back the deploy. If search is still slow 15 minutes later, page the search vendor.' \
  --about checkout --about runbook --about search \
  --because 'INC-312: the rollback did not fix search. The vendor fixed it 20 minutes after the page.'

# triage on the next search slowdown
advance 2d
mem recall --about search --about runbook
