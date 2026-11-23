.PHONY: build test lint coverage sonic-build sonic-smoke sonic-full sonic-examples clean

## build: Compile the workspace
build:
	cargo build --workspace

## test: Run library and CLI tests
test:
	cargo test --workspace

## lint: Format check and clippy
lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings

## coverage: Line coverage must stay at or above 95%
coverage:
	cargo llvm-cov --workspace --fail-under-lines 95

## sonic-build: Release build of the workspace inside gosonic
sonic-build:
	gosonic ci run build

## sonic-smoke: Lint, build, and test inside gosonic
sonic-smoke:
	gosonic ci run smoke-test

## sonic-full: Lint, build, test, coverage, and the example scenarios
sonic-full:
	gosonic ci run full-test

## sonic-examples: Run invoice, incident, and onboarding
sonic-examples:
	gosonic ci run examples

SONIC_CACHE_VOLUMES := gosonic-cache-cargo-home gosonic-cache-cargo-target

## clean: Remove target and the Sonic cargo volumes
clean:
	rm -rf target
	@if command -v docker >/dev/null 2>&1; then \
		for volume in $(SONIC_CACHE_VOLUMES); do \
			if docker volume inspect "$$volume" >/dev/null 2>&1; then \
				docker volume rm "$$volume"; \
			fi; \
		done; \
	fi
