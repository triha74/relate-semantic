#!/usr/bin/env bash
# expect: none
# The first mitigate attempt fails and the agent notes why in the case
# scope. The retry works, so the note is wrong now and is forgotten.
# Forgetting it again changes nothing.
set -euo pipefail
source "$(dirname "$0")/seed.sh"
CASE=case-inc-313

# mitigate failed: rollback refused
mem remember rollback-blocked \
  --claim 'The rollback is refused while the deploy lock is held.' \
  --about checkout --about rollback --scope "$CASE" \
  --because 'mitigate attempt 1 failed with "rollback refused".'

# mitigate retried and resolved
advance 20m
mem recall --about rollback --scope "$CASE"
mem forget rollback-blocked --scope "$CASE" \
  --because 'The lock cleared and the retried rollback stuck.'
mem forget rollback-blocked --scope "$CASE" \
  --because 'The lock cleared and the retried rollback stuck.'

# close-ticket
mem recall --about rollback --scope "$CASE"
