#!/bin/bash
set -eu

GITATTRIBUTES_FILE=".gitattributes"

echo "✅ .gitattributes を生成します"

cat > "$GITATTRIBUTES_FILE" <<EOF
# 秘密ファイルやローカル向けスクリプトは配布物から除外
.env.secret export-ignore
create-secret.sh export-ignore
EOF

echo "📦 $GITATTRIBUTES_FILE を作成しました。GitHubのZIPや git archive では除外されます。"
