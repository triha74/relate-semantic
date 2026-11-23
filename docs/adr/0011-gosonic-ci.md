# ADR-0011: CI is gosonic

Status: Accepted

## Context

Relate and relate-flow run their portable stages from `.sonic/ci.yml` and use GitHub Actions only to build a pinned gosonic and launch one stage per job. Relate Semantic works the same way, including a lane that runs the example scenarios.

## Decision

`.sonic/ci.yml` is the pipeline. Stages are `lint`, `build`, `test`, `coverage`, and `examples`. `build` is `cargo build --workspace --release` with `./target/sonic` bind-mounted at the cargo target directory, so the Linux binary is `target/sonic/release/relate-semantic` and a host `cargo build --release` in `target/release` is left in place. The examples stage is a matrix of `invoice`, `incident`, and `onboarding`, the same names relate-flow uses. Suites are `smoke-test` (lint, build, test), `full-test` (those plus coverage and examples), and `all-examples`. The image is `localhost/relate-semantic-sonic-rust:1.88`, Ubuntu 24.04, Rust 1.88, rustfmt, clippy, and cargo-llvm-cov. The workflow checks out `triha74/gosonic` at `36b8e47f575c2436515e3ba5f4819771131e83ff`, builds it with `CGO_ENABLED=0`, and runs one stage per job. `scripts/sonic-cache-sync.sh` seeds and saves the cargo volumes around `actions/cache`. The cache key includes the stage name.

## Alternatives considered

- Hand-written `cargo` steps in the workflow.
- One shared CI definition for relate-flow and relate-semantic.

## Consequences

The commands live in one file. The Dockerfile, the gosonic action, and the cache script are byte-for-byte relate-flow's. `make sonic-smoke`, `make sonic-full`, and `make sonic-examples` call `gosonic ci run`. `make clean` removes `target` and the `gosonic-cache-cargo-home` and `gosonic-cache-cargo-target` volumes.
