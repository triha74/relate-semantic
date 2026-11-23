#!/usr/bin/env bash
# expect: use
# APP-3 hits the sanctions list through its owner. Compliance declines. The
# hit is about the entity, not the application, so it goes in org and a
# later application with the same owner recalls it at sanctions-screen.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# compliance-review declined
mem remember sanctioned-entity --agent compliance \
  --claim 'Orion Trading LLC is on the EU sanctions list. Decline any application where it is a beneficial owner.' \
  --about orion-trading --about sanctions --scope org \
  --because 'Screening hit on APP-3, confirmed by compliance on 2026-11-10.'

# sanctions-screen for APP-8, owned by Orion Trading LLC
advance 5d
mem recall --about orion-trading --scope case-app-8 --scope org
