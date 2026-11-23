#!/usr/bin/env bash
# expect: use
# An earlier case already remembered the NW-2024 price. verify-pricing uses
# it without opening the agreement. Remembering the same fact again returns
# the stored generation and writes nothing.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
seed_price >/dev/null

# verify-pricing
mem recall --about northwind --about nw-2024 --scope "$CASE" --scope org

# verify-discounts: the 10% discount is on the invoice
seed_price

# pay
mem recall --key widget-price --scope "$CASE" --scope org
