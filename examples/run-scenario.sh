#!/usr/bin/env bash
# Run every examples/<scenario>/run-*.sh in its own store, or the scripts named
# after the scenario. The scripts call target/release/relate-semantic unless
# RELATE_SEMANTIC_BIN is set. -v prints each relate-semantic command and its output.
# -text makes recall print text instead of JSON.
set -euo pipefail

usage() {
  echo "usage: examples/run-scenario.sh [-v|--verbose] [-text|--text] <invoice|incident|onboarding> [run-script...]" >&2
  exit 2
}

verbose=0
text=0
scenario=""
wanted=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--verbose) verbose=1 ;;
    -text|--text) text=1 ;;
    -h|--help) usage ;;
    -*)
      echo "unknown flag: $1" >&2
      usage
      ;;
    *)
      if [[ -z "$scenario" ]]; then
        scenario="$1"
      else
        wanted+=("$1")
      fi
      ;;
  esac
  shift
done
[[ -n "$scenario" ]] || usage

root="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=bin.sh
source "$root/bin.sh"
export RELATE_SEMANTIC_BIN

shopt -s nullglob
scripts=("$root/$scenario"/run-*.sh)
if ((${#scripts[@]} == 0)); then
  echo "no run scripts in examples/$scenario" >&2
  exit 1
fi
if ((${#wanted[@]} > 0)); then
  scripts=()
  for name in "${wanted[@]}"; do
    name="$(basename "$name")"
    [[ "$name" == *.sh ]] || name="$name.sh"
    script="$root/$scenario/$name"
    if [[ ! -f "$script" ]]; then
      echo "no run script examples/$scenario/$name" >&2
      exit 1
    fi
    scripts+=("$script")
  done
fi

fail=0
for script in "${scripts[@]}"; do
  tmp="$(mktemp -d)"
  export RELATE_SEMANTIC_DATABASE="$tmp/db"
  export RELATE_SEMANTIC_NOW="2026-11-10T18:00:00Z"
  expect="$(awk '/^# expect: / { print $3; exit }' "$script")"
  name="$(basename "$script")"
  echo "== $name (expect $expect)"
  if [[ "$verbose" -eq 1 ]]; then
    export RELATE_SEMANTIC_VERBOSE=1
  else
    unset RELATE_SEMANTIC_VERBOSE
  fi
  if [[ "$text" -eq 1 ]]; then
    export RELATE_SEMANTIC_OUTPUT=text
  else
    unset RELATE_SEMANTIC_OUTPUT
  fi
  if [[ "$verbose" -eq 1 ]]; then
    if ! out="$(bash "$script" 2>&1 | tee /dev/stderr)"; then
      echo "FAIL $name (script exited non-zero)" >&2
      fail=1
      rm -rf "$tmp"
      continue
    fi
  elif ! out="$(bash "$script" 2>&1)"; then
    echo "$out" >&2
    echo "FAIL $name (script exited non-zero)" >&2
    fail=1
    rm -rf "$tmp"
    continue
  fi
  if [[ "$text" -eq 1 ]]; then
    action="$(printf '%s\n' "$out" | awk '/^Recall: / { line = $2 } END { print line }' | tr -d '.')"
  else
    last="$(printf '%s\n' "$out" | awk '/^\{/ { line = $0 } END { print line }')"
    action="$(printf '%s' "$last" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("action",""))')"
  fi
  if [[ "$action" != "$expect" ]]; then
    if [[ "$verbose" -eq 0 ]]; then
      echo "$out" >&2
    fi
    echo "FAIL $name: last action is '$action', header expects '$expect'" >&2
    fail=1
  else
    echo "ok $name"
  fi
  rm -rf "$tmp"
done

exit "$fail"
