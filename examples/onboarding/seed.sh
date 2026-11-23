#!/usr/bin/env bash
# Every application starts from the accepted identity documents.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../bin.sh"
example_store
export RELATE_SEMANTIC_AGENT=kyc

mem remember id-documents \
  --claim 'Accept a passport or a national ID card as proof of identity. A driving licence is not enough.' \
  --about identity --about documents --scope org \
  --because 'KYC policy K-4, 2026-02-01.' >/dev/null
