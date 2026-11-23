#!/usr/bin/env bash
# expect: stale
# One chart is stale. The dashboard runbook is 45 days old, and the agent
# asks for nothing older than 30 days, so recall says which claim to check.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
mem remember chart-refresh \
  --claim 'A stale chart is fixed by refreshing the chart cache from the dashboards admin page.' \
  --about dashboards --about runbook --scope org \
  --because 'INC-288: refreshing the cache fixed the revenue chart.' >/dev/null
advance 45d

# triage: sev3, one chart is stale
mem recall --about severity

# mitigate
mem recall --about dashboards --about runbook --fresh 30d
