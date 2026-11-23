#!/usr/bin/env bash
# expect: none
# The person rejects INV-1042. The rejection is about this invoice only, so
# it goes in the case scope. Closing the case prunes the scope.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-discounts
mem recall --about northwind --about discounts --scope "$CASE" --scope org
seed_price

# human-review rejected
mem remember review-outcome \
  --claim 'REV-21 rejected INV-1042. Do not pay it.' \
  --about inv-1042 --scope "$CASE" \
  --because 'human-review result rejected on REV-21.'

# close-rejected
mem recall --key review-outcome --scope "$CASE"
mem prune --scope "$CASE"
mem recall --key review-outcome --scope "$CASE"
