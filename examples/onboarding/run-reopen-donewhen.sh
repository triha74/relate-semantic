#!/usr/bin/env bash
# expect: none
# collect-docs is reopened to require a proof of address. Compliance changes
# what counts as proof, and the subjects change with it. A recall by the old
# subject finds nothing, so stale wording cannot leak back.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
export RELATE_SEMANTIC_AGENT=compliance

mem remember address-proof \
  --claim 'A utility bill under 3 months old is proof of address.' \
  --about address --about documents --scope org \
  --because 'Compliance memo CM-3.'

# revise: collect-docs needs a proof of address
mem recall --about address --scope case-app-5 --scope org

advance 1d
mem correct address-proof --scope org \
  --claim 'A utility bill or a bank statement under 3 months old is proof of address.' \
  --about proof-of-address --about documents \
  --because 'Compliance memo CM-4 replaced CM-3.'

# collect-docs again
mem recall --about address --scope case-app-5 --scope org
