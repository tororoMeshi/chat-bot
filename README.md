# chat-bot

Discord 向けのチャットボットです。`/q` プレフィックスで入力された内容を Gemini に送信し、会話履歴と設定プロンプトを加味した返信を返します。

## 構成

- `chat-bot/` Rust 実装本体
- `chat-bot/prompt_q.md` プロンプトテンプレート
- `yaml/` Kubernetes マニフェスト
- `create-secret.sh` / `create-configmap.sh` Kubernetes 用の Secret/ConfigMap 作成

## 主要機能

- `/q <質問>` のプレフィックスコマンド
- 直近の履歴を取得してプロンプトに付与
- Gemini API からのエラー本文をログ出力
- 429 の場合はユーザー向けメッセージを送信

## 必要な環境変数

- `DISCORD_TOKEN` Discord Bot トークン
- `GEMINI_API_KEY` Gemini API キー
- `GEMINI_MODEL` Gemini モデル名（未設定時は `gemini-pro`）
- `CHAT_HISTORY_LIMIT` 履歴の取得件数（未設定時は `3`）
- `RUST_LOG` ログレベル（例: `info`）

## ローカル実行

```bash
cd chat-bot
cargo run
```

## Lint

`chat-bot/lint.sh` は Docker 上で `fmt`/`clippy`/`outdated` と Trivy を実行します。

```bash
cd chat-bot
./lint.sh
```

## Docker ビルド

```bash
cd chat-bot
docker build -t tororomeshi/chat-bot:latest .
```

## Kubernetes デプロイ

Secret と ConfigMap を用意してから `yaml/deploy.yaml` を適用します。

```bash
./create-secret.sh
./create-configmap.sh
kubectl apply -f yaml/deploy.yaml
```

`yaml/deploy.yaml` の `GEMINI_MODEL` を環境に合わせて変更してください。

## トラブルシューティング

- `Gemini returned an error status: 429` が出る場合は Gemini のクォータ超過です。時間をおいて再試行するか、プランとクォータを確認してください。
