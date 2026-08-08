# chat-bot

友人同士の小規模なDiscordサーバー向けBotです。Bot専用チャンネルで `/q <質問>` を受け取り、そのチャンネルの直近メッセージと質問をGeminiへ送り、回答をDiscordへ返します。複数サーバー向けの汎用Botではありません。

## 動作とデータの扱い

- `ALLOWED_CHANNEL_ID` で指定した1つのBot専用チャンネルだけで動作します。
- 指定チャンネルの直近メッセージが会話コンテキストとしてGeminiへ送信されます。機密情報を書き込むチャンネルでは使わないでください。
- Bot自身はDBなどへ会話履歴を永続保存しません。Discord上の履歴だけを参照するステートレス構成です。
- 複数人の会話ではDiscord表示名を話者識別に使います。表示名はサーバーニックネーム、グローバル表示名、usernameの順で解決します。
- Gemini APIでは、人格・応答ルール・会話形式を`systemInstruction`へ、Discord履歴・現在の話者・現在の質問を`contents`へ分離して送信します。
- ユーザーごとの5秒クールダウンと最大2件の同時実行制限はプロセス内だけで保持します。再起動するとクールダウン状態は消えます。
- 質問は1,500文字、Discordへの返信は1,900文字までに制限します。
- GeminiへのHTTPリクエストは約20秒でタイムアウトします。

## 構成

- `chat-bot/`: Rust実装、Dockerfile、バイナリへ埋め込むプロンプト
- `yaml/deploy.yaml`: Kubernetes Deployment
- `create-secret.sh`: `.env.secret` からKubernetes Secretをapplyするスクリプト

## 必要な環境変数

すべて起動時に必須です。未設定、空文字、不正値の場合は起動に失敗します。コードとマニフェストにモデル名の暗黙デフォルトはありません。

| 変数 | 内容 |
| --- | --- |
| `DISCORD_TOKEN` | Discord Botトークン |
| `GEMINI_API_KEY` | Gemini APIキー |
| `GEMINI_MODEL` | 利用するGeminiモデル名 |
| `ALLOWED_CHANNEL_ID` | Botの利用を許可するDiscordチャンネルID（1件） |
| `CHAT_HISTORY_LIMIT` | Geminiへ渡す履歴件数（1〜20） |
| `RUST_LOG` | ログフィルター（例: `info`） |

## ローカル実行

リポジトリ直下の `.env`、またはシェル環境へ必要な値を設定します。

```dotenv
DISCORD_TOKEN=replace_me
GEMINI_API_KEY=replace_me
GEMINI_MODEL=gemini-2.5-flash-lite
ALLOWED_CHANNEL_ID=123456789012345678
CHAT_HISTORY_LIMIT=5
RUST_LOG=info
```

```bash
cd chat-bot
cargo run
```

Discord Developer PortalではMessage Content Intentを有効にし、Botには対象チャンネルの閲覧、履歴閲覧、メッセージ送信に必要な権限だけを付与してください。

## Format、Lint、テスト

```bash
cd chat-bot
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

`chat-bot/lint.sh` はDocker上でfmt、check、test、clippyを実行し、その後にTrivyによるイメージスキャンを行います。

## Dockerビルド

`latest` は使わず、GitコミットSHAやリリース番号など変更しないタグを指定します。

```bash
cd chat-bot
IMAGE_TAG=$(git rev-parse --short HEAD)
docker build -t "tororomeshi/chat-bot:${IMAGE_TAG}" .
```

Docker Hubへ送る場合も固定タグを明示します。スクリプトは `latest` を作成しません。

```bash
./push_to_dockerhub.sh "$IMAGE_TAG"
```

`chat-bot/prompt_q.md` は `include_str!` でバイナリへ埋め込まれます。プロンプトを変更した場合はイメージを再ビルドしてください。

## Kubernetesデプロイ

1. namespaceを用意します。
2. `./generate-env.sh` などでGit管理外の `.env.secret` に `GEMINI_API_KEY` と `DISCORD_TOKEN` を設定します。
3. `./create-secret.sh` を実行します。既存Secretも削除せずapplyで更新されます。
4. `yaml/deploy.yaml` の `SET_IMAGE_TAG` をDockerで付けた固定タグへ、`SET_DISCORD_CHANNEL_ID` をBot専用チャンネルIDへ置換します。必要なら `GEMINI_MODEL` と `CHAT_HISTORY_LIMIT` も更新します。
5. Deploymentを適用します。

```bash
kubectl create namespace chat-bot
./create-secret.sh
kubectl apply -f yaml/deploy.yaml
```

このBotはHTTPサーバーを持たないため、Service、containerPort、HTTPヘルスチェックは不要です。
