#!/usr/bin/env bash
# new-rust.sh: Docker 上で cargo new を実行し、
#               lint.sh, push_to_dockerhub.sh, Dockerfile を自動生成するスクリプト
# Usage: ./new-rust.sh <project-name>
# Optionally set DOCKERHUB_USER 環境変数（デフォルト: tororomeshi）

set -euo pipefail

DOCKERHUB_USER="${DOCKERHUB_USER:-tororomeshi}"

if [ $# -ne 1 ]; then
  echo "Usage: $0 <project-name>" >&2
  exit 1
fi

PROJECT_NAME="$1"
PROJECT_DIR="$(pwd)/$PROJECT_NAME"

if [ -e "$PROJECT_DIR" ]; then
  echo "Error: directory '$PROJECT_DIR' already exists." >&2
  exit 1
fi

USER_ID=$(id -u)
GROUP_ID=$(id -g)

docker run --rm \
  -u "${USER_ID}:${GROUP_ID}" \
  -v "$(pwd)":/opt \
  -w /opt \
  rust:1.86 \
  cargo new "$PROJECT_NAME" --vcs none

echo "✅ Created new Rust project at $PROJECT_DIR"

cd "$PROJECT_DIR"

# lint.sh の生成
cat <<EOF > lint.sh
#!/usr/bin/env bash
set -euo pipefail

LINT_IMAGE=rust-lint-extended

docker build -t "\$LINT_IMAGE" - << 'DOCKERFILE'
FROM rust:1.86

RUN rustup component add rustfmt clippy && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev libwebp-dev git curl && \
    cargo install cargo-outdated && \
    rm -rf /var/lib/apt/lists/*
DOCKERFILE

docker run --rm \
  -v "\$PWD":/usr/src/app \
  -w /usr/src/app \
  "\$LINT_IMAGE" bash -c "
    cargo fmt --all &&
    cargo check &&
    cargo clippy -- -D warnings &&
    cargo outdated || true
  "

APP_IMAGE="${DOCKERHUB_USER}/${PROJECT_NAME}"
docker build -t "\${APP_IMAGE}:lint-temp" .

docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "\${HOME}/.cache/trivy":/root/.cache/trivy \
  aquasec/trivy:latest image \
    --exit-code 1 \
    --severity CRITICAL,HIGH \
    "\${APP_IMAGE}:lint-temp"

docker rmi "\${APP_IMAGE}:lint-temp" || true
EOF

chmod +x lint.sh
chown "${USER_ID}:${GROUP_ID}" lint.sh
echo "✅ lint.sh generated"

# push_to_dockerhub.sh の生成
cat <<EOF > push_to_dockerhub.sh
#!/usr/bin/env bash
set -euo pipefail

DOCKERHUB_USER="\${DOCKERHUB_USER:-${DOCKERHUB_USER}}"
IMAGE_TAG=\$(date +%Y%m%d%H%M)
if [ \$# -ge 1 ]; then IMAGE_TAG="\$1"; fi

IMAGE_NAME="\${DOCKERHUB_USER}/${PROJECT_NAME}"

SCRIPT_DIR=\$(cd "\$(dirname "\$0")" && pwd)
cd "\$SCRIPT_DIR"

if [ -f "Cargo.lock" ]; then
  echo "Removing Cargo.lock..."
  rm Cargo.lock
fi

echo "Building Docker image..."
docker build -t "\${IMAGE_NAME}:\${IMAGE_TAG}" -t "\${IMAGE_NAME}:latest" .

push_image() {
  local TAG="\$1"
  echo "Pushing \${IMAGE_NAME}:\${TAG}..."
  if ! docker push "\${IMAGE_NAME}:\${TAG}"; then
    echo "Docker push failed for tag \${TAG}. Please run 'docker login'." >&2
    exit 1
  fi
}

push_image "\${IMAGE_TAG}"
push_image "latest"

echo "✅ Docker image pushed: \${IMAGE_NAME}:\${IMAGE_TAG} (also tagged latest)"
EOF

chmod +x push_to_dockerhub.sh
chown "${USER_ID}:${GROUP_ID}" push_to_dockerhub.sh
echo "✅ push_to_dockerhub.sh generated"

# Dockerfile の生成
cat <<EOF > Dockerfile
FROM rust:1.86 AS builder
WORKDIR /app

COPY Cargo.toml ./
RUN mkdir src \
    && echo 'fn main() { println!("hello"); }' > src/main.rs \
    && cargo fetch \
    && rm -rf src

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/base-nossl-debian12:nonroot
COPY --from=builder /app/target/release/${PROJECT_NAME} /usr/local/bin/${PROJECT_NAME}
ENTRYPOINT ["/usr/local/bin/${PROJECT_NAME}"]
EOF

chown "${USER_ID}:${GROUP_ID}" Dockerfile
echo "✅ Dockerfile generated"
