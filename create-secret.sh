#!/usr/bin/env bash
set -euo pipefail

NAMESPACE="chat-bot"
SECRET_NAME="chat-bot-secrets"
ENV_FILE=".env.secret"

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ $ENV_FILE が見つかりません"
  exit 1
fi

set -a
# shellcheck disable=SC1090
. "./$ENV_FILE"
set +a

if [ -z "${GEMINI_API_KEY:-}" ] || [ -z "${DISCORD_TOKEN:-}" ]; then
  echo "❌ GEMINI_API_KEY と DISCORD_TOKEN を $ENV_FILE に設定してください"
  exit 1
fi

kubectl create secret generic "$SECRET_NAME" \
  --from-literal=gemini_api_key="$GEMINI_API_KEY" \
  --from-literal=discord_token="$DISCORD_TOKEN" \
  --namespace="$NAMESPACE" \
  --dry-run=client \
  -o yaml |
  kubectl apply -f -

echo "✅ Secret '$SECRET_NAME' を namespace '$NAMESPACE' に反映しました"
