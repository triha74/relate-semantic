#!/usr/bin/env bash
# expect: use
# Northwind is not on the approved vendor list. The person approves payment
# anyway, so the vendor memory is corrected, not replaced. log shows both.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-vendor
mem recall --about northwind --about vendor-list --scope "$CASE" --scope org
mem remember vendor-status \
  --claim 'Northwind is not on the approved vendor list.' \
  --about northwind --about vendor-list --scope org \
  --because 'Checked the approved vendor list for INV-1042 on 2026-11-10.'

# human-review approved
advance 2h
mem correct vendor-status --scope org --agent human \
  --claim 'Northwind is not on the approved vendor list. A person approves each Northwind invoice before payment.' \
  --because 'REV-22 approved paying INV-1042 anyway.'
mem log vendor-status --json

# verify-vendor on the next Northwind invoice
advance 3d
mem recall --about northwind --about vendor-list
