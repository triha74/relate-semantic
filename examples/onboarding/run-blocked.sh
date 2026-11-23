#!/usr/bin/env bash
# expect: use
# The document image for APP-4 is unreadable. The agent first tries to paste
# the whole OCR dump as a claim and gets too_long, then uses a subject that
# is not a slug and gets bad_slug. It follows the fix and writes the fact.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
CASE=case-app-4
because='verify-identity blocked: the document image is unreadable.'
dump="OCR: $(printf 'P<UTO#### ?? unreadable ## %.0s' {1..30})"

# verify-identity blocked
refused too_long remember app-4-document --claim "$dump" \
  --about passport-scan --scope "$CASE" --because "$because"
refused bad_slug remember app-4-document \
  --claim 'The passport scan for APP-4 is unreadable. Ask the applicant for a new scan.' \
  --about 'Passport Scan' --scope "$CASE" --because "$because"
mem remember app-4-document \
  --claim 'The passport scan for APP-4 is unreadable. Ask the applicant for a new scan.' \
  --about passport-scan --scope "$CASE" --because "$because"

# revise and commit, then verify-identity again
advance 1d
mem recall --about passport-scan --scope "$CASE"
