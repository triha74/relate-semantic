#!/usr/bin/env bash
# expect: use
# Checkout is down. triage uses the severity guide. mitigate finds no
# runbook, the fix works, and postmortem remembers it with the incident as
# the source. The next checkout outage recalls it.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
CASE=case-inc-311

# triage
mem recall --about severity --scope "$CASE" --scope org

# mitigate
mem recall --about checkout --about runbook --scope "$CASE" --scope org

# postmortem PM-14 filed
mem remember checkout-runbook --claim "$RUNBOOK" \
  --about checkout --about runbook --scope org --because "$RUNBOOK_SRC"

# Ten days later, mitigate on the next checkout outage
advance 10d
mem recall --about checkout --about runbook --scope case-inc-320 --scope org
