#!/usr/bin/env bash
set -euo pipefail

DOCKERHUB_USER="${DOCKERHUB_USER:-tororomeshi}"
IMAGE_TAG="${1:?Usage: $0 <immutable-image-tag>}"
IMAGE_NAME="${DOCKERHUB_USER}/chat-bot"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

cd "$SCRIPT_DIR"

echo "Building ${IMAGE_NAME}:${IMAGE_TAG}..."
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

echo "Pushing ${IMAGE_NAME}:${IMAGE_TAG}..."
if ! docker push "${IMAGE_NAME}:${IMAGE_TAG}"; then
  echo "Docker push failed. Please run 'docker login'." >&2
  exit 1
fi

echo "✅ Docker image pushed: ${IMAGE_NAME}:${IMAGE_TAG}"
