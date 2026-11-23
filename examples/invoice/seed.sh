#!/usr/bin/env bash
# Every invoice case starts from one org-wide rule. seed_price writes the
# NW-2024 price the way an earlier case would have.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../bin.sh"
example_store
export RELATE_SEMANTIC_AGENT=invoices

CASE=case-inv-1042
PRICE='Under NW-2024, widgets are $100, with 10% off a widget line over $5000.'
PRICE_SRC='Read from NW-2024 section 3 while checking INV-1042.'

mem remember review-threshold \
  --claim 'A person reviews any invoice whose billed total differs from the agreed total.' \
  --about invoices --about review --scope org \
  --because 'Finance policy FP-7, effective 2026-01-01.' >/dev/null

seed_price() {
  mem remember widget-price --claim "$PRICE" \
    --about northwind --about nw-2024 --about discounts --scope org \
    --because "$PRICE_SRC"
}
