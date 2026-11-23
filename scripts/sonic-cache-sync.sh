#!/usr/bin/env bash
# Copy host cargo trees into the Sonic cache volumes, and back.
# Gosonic names a cache volume gosonic-cache-<source>.
# Usage: sonic-cache-sync.sh seed|save <stage-subdir>
set -euo pipefail

mode="${1:?usage: sonic-cache-sync.sh seed|save <subdir>}"
subdir="${2:?usage: sonic-cache-sync.sh seed|save <subdir>}"
if [[ "$mode" != "seed" && "$mode" != "save" ]]; then
  echo "sonic-cache-sync: mode must be seed or save" >&2
  exit 1
fi
if [[ ! "$subdir" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "sonic-cache-sync: refusing subdir $subdir" >&2
  exit 1
fi
if ! command -v docker >/dev/null 2>&1; then
  echo "sonic-cache-sync: docker is not installed; skipping" >&2
  exit 0
fi

prefix="${SONIC_CACHE_VOLUME_PREFIX:-gosonic-cache}"
home_volume="${prefix}-cargo-home"
target_volume="${prefix}-cargo-target"
cargo_home="${CARGO_HOME:-${HOME}/.cargo}"
host_target="${GITHUB_WORKSPACE:-$(pwd)}/target"
image="${SONIC_CACHE_SYNC_IMAGE:-ubuntu:24.04}"

docker volume create "$home_volume" >/dev/null
docker volume create "$target_volume" >/dev/null

copy() {
  local volume="$1" rel="$2" host="$3" direction="$4"
  mkdir -p "$host"
  if [[ "$direction" == "in" ]]; then
    docker run --rm \
      -v "$volume":/vol \
      -v "$host":/host:ro \
      "$image" \
      bash -c "mkdir -p /vol/$rel && cp -a /host/. /vol/$rel/"
  else
    docker run --rm \
      -v "$volume":/vol:ro \
      -v "$host":/host \
      "$image" \
      bash -c "mkdir -p /host && if [ -d /vol/$rel ]; then cp -a /vol/$rel/. /host/; fi"
  fi
}

if [[ "$mode" == "seed" ]]; then
  [[ -d "$cargo_home/registry" ]] && copy "$home_volume" registry "$cargo_home/registry" in
  [[ -d "$cargo_home/git" ]] && copy "$home_volume" git "$cargo_home/git" in
  [[ -d "$host_target" ]] && copy "$target_volume" "$subdir" "$host_target" in
else
  copy "$home_volume" registry "$cargo_home/registry" out
  copy "$home_volume" git "$cargo_home/git" out
  copy "$target_volume" "$subdir" "$host_target" out
fi
