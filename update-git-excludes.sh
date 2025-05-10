#!/bin/bash
set -eu

ENV_FILE=".env.secret"

# .gitignore に .env.secret を追記（なければ）
if [ ! -f .gitignore ] || ! grep -Fxq "$ENV_FILE" .gitignore; then
  echo "$ENV_FILE" >> .gitignore
  echo "✅ .gitignore に $ENV_FILE を追加しました"
else
  echo "✅ .gitignore は既に設定済みです"
fi

# .gitattributes に export-ignore を追記（なければ）
if [ ! -f .gitattributes ]; then
  touch .gitattributes
fi

if ! grep -Fxq "$ENV_FILE export-ignore" .gitattributes; then
  echo "$ENV_FILE export-ignore" >> .gitattributes
  echo "✅ .gitattributes に $ENV_FILE export-ignore を追加しました"
else
  echo "✅ .gitattributes は既に設定済みです"
fi
