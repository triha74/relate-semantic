#!/usr/bin/env bash
# expect: narrow
# Mitigate fails twice and the plan is revised to bring the vendor in. The
# agent widens its recall to every checkout memory, gets more than the limit,
# and is shown the keys to choose from.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
seed_runbook >/dev/null
note() {
  mem remember "$1" --claim "$2" --about checkout --about "$3" --scope org \
    --because 'Checkout service catalog, 2026-11-01.' >/dev/null
}
note checkout-owners 'The payments team owns checkout. Their escalation alias is payments-oncall.' owners
note checkout-dashboards 'Checkout latency and error rate are on the checkout-golden dashboard.' dashboards
note checkout-vendor 'The checkout payment gateway vendor is paged through the vendor portal, priority P1.' vendor
note checkout-flags 'The checkout-v2 flag can be turned off without a deploy.' flags
note checkout-db 'Checkout writes go to the orders-primary database.' database

# mitigate failed twice
mem recall --key checkout-runbook --scope org

# revise: mitigate with the vendor on the call
mem recall --about checkout
