#!/usr/bin/env bash
# expect: use
# The discount is missing. The agent reads NW-2024 once and remembers the
# price, and remembers how the person handled the shortfall. The next
# Northwind invoice recalls both instead of reading the agreement again.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-discounts
mem recall --about northwind --about discounts --scope "$CASE" --scope org
seed_price

# human-review approved $9,000
mem remember shortfall-handling \
  --claim 'When a Northwind invoice omits the NW-2024 discount, pay the agreed total, not the billed total.' \
  --about northwind --about discounts --about shortfall --scope org \
  --because 'REV-19 approved $9,000 for INV-1042 on 2026-11-10.'

# A week later: verify-discounts for INV-1077
advance 7d
mem recall --about northwind --about discounts --scope case-inv-1077 --scope org
