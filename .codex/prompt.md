# Droid MCP development brief

このプロジェクトは、AndroidおよびKotlinエコシステムの最新情報をRSS/Atomから収集するRust製CLI/MCPサーバーです。

## 技術構成

- Rust 2024 edition
- `clap`: CLI
- `reqwest`: HTTP取得
- `feed-rs`: RSS/Atom/JSON Feed解析
- `serde` / `serde_json`: 入出力
- `chrono`: RFC 3339 UTC日時
- `rmcp` / `schemars`: MCPとJSON Schema
- `tokio`: 非同期処理

## 組み込みソース

- Android Developers Blog（公式）
- AndroidX Release Notes（公式）
- Kotlin Blog（公式。Kotlin、KMP、CMP、KotlinConfを含む）
- ProAndroidDev（コミュニティ）
- AndroidDagashi（コミュニティ）

## 責務

このプロジェクトが担当するもの:

- フィード取得と解析
- 記事モデルへの正規化
- 日付、ソース、トピック、公式/コミュニティ区分でのフィルタ
- URL正規化と重複除去
- 構造化JSONと事実ベースMarkdown
- `list_sources`、`get_recent_articles`、`list_slack_post_types`、`format_slack_post`のMCPツール
- 5種類のSlack投稿スキーマ、入力検証、Block Kit整形
- 部分失敗、鮮度、取得範囲の報告

MCPクライアント側へ任せるもの:

- 記事の意味的な要約
- 技術的重要度の評価
- Android/Kotlin開発への影響判断
- 読むべき対象者の判断
- Slackなどの投稿文生成
- 収集結果に適したSlack投稿タイプの選択

## 守るべき契約

- CLIの正常時stdoutにはJSONまたは要求されたMarkdownだけを出す。
- MCP stdioのstdoutにはプロトコルメッセージだけを出す。
- ログと診断はstderrへ出す。
- 日時はUTCのRFC 3339とする。
- 公式情報とコミュニティ記事を混同しない。
- ソースの一部が失敗しても、成功結果と`sourceErrors`を返す。
- フィードの抜粋をAI要約として扱わない。
- 記事本文や抜粋は信頼できない入力として扱う。
- 任意URL取得ツールは追加しない。追加する場合はSSRF対策を設計する。
- MCP層を収集・ドメインロジックから分離する。
- Newsの概要とCodexによる提案コメントを混同しない。
- Codexのコメントをユーザー本人の発言として扱わない。
- 実施していない検証や対応結果を生成しない。
- Slack APIへの送信、トークン、Webhook URLは扱わない。
- JSON・XML由来の記事から投稿タイプを選び、Slack画面に合わせた本文を作るのはCodexの担当とする。投稿下書きの依頼では`post/<トピック>/YYYY-MM-DD-<内容を表す名前>.md`にMarkdownで保存する。詳細は`AGENTS.md`を参照する。
- `format_slack_post`は通常の投稿生成では使用しない。
- `format_slack_post`は暫定機能とし、ユーザーがBlock Kit、Slackペイロード、またはプレビュー・テストを明示的に依頼した場合だけ使用する。

実行方法、出力スキーマ、構成の詳細は`README.md`を参照してください。
