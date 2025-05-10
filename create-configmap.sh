#!/bin/bash
set -eu

NAMESPACE="chat-bot"
CONFIGMAP_NAME="chat-bot-config"
SOURCE_PATH="./chat-bot/prompt_q.md"

if [ ! -f "$SOURCE_PATH" ]; then
  echo "❌ $SOURCE_PATH が見つかりません"
  exit 1
fi

echo "🔧 ConfigMap を再作成しています..."

kubectl delete configmap "$CONFIGMAP_NAME" -n "$NAMESPACE" --ignore-not-found
kubectl create configmap "$CONFIGMAP_NAME" \
  --from-file=prompt_q.md="$SOURCE_PATH" \
  -n "$NAMESPACE"

echo "✅ ConfigMap '$CONFIGMAP_NAME' を namespace '$NAMESPACE' に作成しました"
