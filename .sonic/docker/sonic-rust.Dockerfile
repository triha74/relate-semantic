# Local CI image. Built by the Sonic before action. Not published.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        gcc \
        git \
        pkg-config \
        python3 \
    && rm -rf /var/lib/apt/lists/* \
    && git config --system --add safe.directory /workspace

# Rust 1.88.0 matches rust-toolchain.toml. The toolchain lives outside
# /root so a stage running as the host uid can execute it. /cargo-home
# and /cargo-target are the cache mounts; mode 1777 lets that uid write
# a volume Docker initialized as root.
RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain 1.88.0 --profile minimal \
            --component rustfmt,clippy,llvm-tools-preview \
    && cargo install cargo-llvm-cov --locked \
    && chmod -R a+rX /usr/local/rustup /usr/local/cargo \
    && mkdir -p /cargo-home /cargo-target \
    && chmod 1777 /cargo-home /cargo-target \
    && rustc --version \
    && cargo llvm-cov --version
