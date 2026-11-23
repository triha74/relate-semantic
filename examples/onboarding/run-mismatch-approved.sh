#!/usr/bin/env bash
# expect: use
# APP-2's name does not match the document. Compliance approves it as a
# transliteration. The ruling applies to every later applicant, so it goes
# in org, and the next mismatch recalls it before asking compliance again.
set -euo pipefail
source "$(dirname "$0")/seed.sh"

# verify-identity: mismatch
mem recall --about name-mismatch --scope case-app-2 --scope org

# compliance-review approved
mem remember transliterated-names --agent compliance \
  --claim 'A name that differs from the document only by transliteration, such as Aleksandr and Alexander, is approved when the date of birth and document number match.' \
  --about name-mismatch --about transliteration --scope org \
  --because 'Compliance approved APP-2 with the corrected name on 2026-11-10.'

# verify-identity for APP-7: mismatch again
advance 2d
mem recall --about name-mismatch --scope case-app-7 --scope org
