#!/usr/bin/env bash
# expect: use
# APP-1 is clear. risk-score remembers the threshold the analyst used, and
# the next applicant is scored the same way.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-identity
mem recall --about identity --scope case-app-1 --scope org

# risk-score
mem recall --about risk --about threshold --scope case-app-1 --scope org
mem remember low-risk-threshold \
  --claim 'Score an applicant low risk when identity is verified, sanctions are clear, and expected monthly volume is under $10,000.' \
  --about risk --about threshold --scope org \
  --because 'Analyst rule applied to APP-1 on 2026-11-10.'

# risk-score for APP-6 the next day
advance 1d
mem recall --about risk --about threshold --scope case-app-6 --scope org
