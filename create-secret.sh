#!/bin/bash
set -eu

NAMESPACE=chat-bot
SECRET_NAME=chat-bot-secrets

# .env.secret が存在しない場合は終了
if [ ! -f .env.secret ]; then
  echo ".env.secret ファイルが見つかりません"
  exit 1
fi

# .env.secret を読み込み
if ! source .env.secret; then
  echo "❌ .env.secret の読み込みに失敗しました"
  exit 1
fi

if [ -z "${GEMINI_API_KEY:-}" ] || [ -z "${DISCORD_TOKEN:-}" ]; then
  echo "❌ GEMINI_API_KEY または DISCORD_TOKEN が未設定です"
  exit 1
fi

# Secret 作成または更新
kubectl delete secret "$SECRET_NAME" -n "$NAMESPACE" --ignore-not-found
kubectl create secret generic "$SECRET_NAME" \
  --from-literal=gemini_api_key="$GEMINI_API_KEY" \
  --from-literal=discord_token="$DISCORD_TOKEN" \
  -n "$NAMESPACE"

echo "✅ Secret '$SECRET_NAME' が namespace '$NAMESPACE' に作成されました"
