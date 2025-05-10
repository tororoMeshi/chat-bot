#!/usr/bin/env bash
set -euo pipefail

LINT_IMAGE=rust-lint-extended

docker build -t "$LINT_IMAGE" - << 'DOCKERFILE'
FROM rust:1.86

RUN rustup component add rustfmt clippy &&     apt-get update &&     apt-get install -y --no-install-recommends       pkg-config libssl-dev libwebp-dev git curl &&     cargo install cargo-outdated &&     rm -rf /var/lib/apt/lists/*
DOCKERFILE

docker run --rm   -v "$PWD":/usr/src/app   -w /usr/src/app   "$LINT_IMAGE" bash -c "
    cargo fmt --all &&
    cargo check &&
    cargo clippy -- -D warnings &&
    cargo outdated || true
  "

APP_IMAGE="tororomeshi/chat-bot"
docker build -t "${APP_IMAGE}:lint-temp" .

docker run --rm   -v /var/run/docker.sock:/var/run/docker.sock   -v "${HOME}/.cache/trivy":/root/.cache/trivy   aquasec/trivy:latest image     --exit-code 1     --severity CRITICAL,HIGH     "${APP_IMAGE}:lint-temp"

docker rmi "${APP_IMAGE}:lint-temp" || true
