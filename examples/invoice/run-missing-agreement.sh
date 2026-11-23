#!/usr/bin/env bash
# expect: conflict
# NW-2024 is not on file, so the agent stores a phone quote for this case
# only. When NW-2024 arrives, the price belongs in org. The same key is now
# live in two scopes, and a recall that names no scope says so.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-pricing, blocked: NW-2024 is not on file
mem recall --about northwind --about nw-2024
mem remember widget-price \
  --claim 'For INV-1042 only, finance quoted $100 per widget by phone while NW-2024 was missing.' \
  --about northwind --about nw-2024 --scope "$CASE" \
  --because 'Phone call with finance on 2026-11-10. NW-2024 was not on file.'

# revise and commit: NW-2024 is now on file
advance 1h
refused key_taken remember widget-price --claim "$PRICE" \
  --about northwind --about nw-2024 --about discounts --scope "$CASE" \
  --because "$PRICE_SRC"
seed_price

# verify-pricing again, without saying which scope wins
mem recall --about northwind --about nw-2024
