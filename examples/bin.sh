# Resolve the relate-semantic binary. Source this; do not execute it.
# RELATE_SEMANTIC_BIN overrides the default, which is the host release build
# at <repo>/target/release/relate-semantic. The gosonic build stage writes a
# Linux binary to target/sonic/release/relate-semantic and does not replace this one.
examples_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$examples_dir/.." && pwd)"
: "${RELATE_SEMANTIC_BIN:=$repo/target/release/relate-semantic}"
if [[ ! -x "$RELATE_SEMANTIC_BIN" ]]; then
  echo "relate-semantic binary is not executable at $RELATE_SEMANTIC_BIN" >&2
  echo "build it with: cargo build --release -p relate-semantic" >&2
  exit 1
fi

# A seed script executed on its own, or a run script that did not get a store
# from run-scenario.sh, uses <scenario>/.relate-semantic and starts empty.
# A caller that set RELATE_SEMANTIC_DATABASE keeps that store.
example_store() {
  if [[ -n "${RELATE_SEMANTIC_DATABASE:-}" ]]; then
    return 0
  fi
  local seed="${BASH_SOURCE[1]}"
  local dir
  dir="$(cd "$(dirname "$seed")" && pwd)"
  export RELATE_SEMANTIC_DATABASE="$dir/.relate-semantic"
  rm -rf "$RELATE_SEMANTIC_DATABASE"
  echo "store: $RELATE_SEMANTIC_DATABASE" >&2
}

# Every example starts at the same moment, so the output is the same on every run.
: "${RELATE_SEMANTIC_NOW:=2026-11-10T18:00:00Z}"
export RELATE_SEMANTIC_NOW

# Call relate-semantic.
# RELATE_SEMANTIC_VERBOSE=1 prints the command on stderr before the output.
# RELATE_SEMANTIC_OUTPUT=text adds --output text to recall.
mem() {
  local args=("$@")
  if [[ "${RELATE_SEMANTIC_OUTPUT:-}" == "text" && "$1" == "recall" ]]; then
    args+=(--output text)
  fi
  if [[ -n "${RELATE_SEMANTIC_VERBOSE:-}" ]]; then
    printf '+' >&2
    printf ' %q' "$RELATE_SEMANTIC_BIN" "${args[@]}" >&2
    printf '\n' >&2
  fi
  "$RELATE_SEMANTIC_BIN" "${args[@]}"
}

# Move the example clock forward. Usage: advance 45d (also h, m, s).
advance() {
  RELATE_SEMANTIC_NOW="$(python3 - "$RELATE_SEMANTIC_NOW" "$1" <<'PY'
import datetime, re, sys
now = datetime.datetime.strptime(sys.argv[1], "%Y-%m-%dT%H:%M:%SZ")
m = re.fullmatch(r"(\d+)([dhms])", sys.argv[2])
if not m:
    sys.exit(f"advance: cannot read {sys.argv[2]!r}; use a number and d, h, m, or s")
unit = {"d": "days", "h": "hours", "m": "minutes", "s": "seconds"}[m.group(2)]
print((now + datetime.timedelta(**{unit: int(m.group(1))})).strftime("%Y-%m-%dT%H:%M:%SZ"))
PY
)"
  export RELATE_SEMANTIC_NOW
  if [[ -n "${RELATE_SEMANTIC_VERBOSE:-}" ]]; then
    echo "+ now $RELATE_SEMANTIC_NOW" >&2
  fi
}

# Show a refusal without stopping set -e, and check its code.
# Usage: refused CODE ARGS...
refused() {
  local want="$1"
  shift
  local out status=0
  out="$(mem "$@")" || status=$?
  printf '%s\n' "$out"
  if [[ "$status" -eq 0 ]]; then
    echo "expected $want, but the command succeeded" >&2
    return 1
  fi
  local got
  got="$(printf '%s' "$out" | python3 -c 'import json,sys; print(json.load(sys.stdin)["error"]["code"])')"
  if [[ "$got" != "$want" ]]; then
    echo "expected $want, got $got" >&2
    return 1
  fi
}
