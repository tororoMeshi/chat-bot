#!/bin/bash
set -eu

ENV_FILE=".env.secret"

if [ -f "$ENV_FILE" ]; then
  echo "⚠️  $ENV_FILE は既に存在します。上書きしますか？ [y/N]"
  read -r answer
  case "$answer" in
    [yY][eE][sS]|[yY]) ;;
    *) echo "キャンセルしました"; exit 0 ;;
  esac
fi

echo "🔐 Gemini API キーを入力してください:"
read -r -p "GEMINI_API_KEY=" GEMINI_API_KEY

echo "🤖 Discord ボットトークンを入力してください:"
read -r -p "DISCORD_TOKEN=" DISCORD_TOKEN

cat > "$ENV_FILE" <<EOF
GEMINI_API_KEY=$GEMINI_API_KEY
DISCORD_TOKEN=$DISCORD_TOKEN
EOF

echo "✅ $ENV_FILE を作成しました（Git に含めないように注意してください）"
