# Droid MCP

AndroidとKotlinエコシステムの最新情報をRSS/Atomから収集し、CLIまたはModel Context Protocol（MCP）から利用するRustアプリケーションです。

収集と正規化だけを担当し、要約、技術的重要度の判断、共有文の生成はMCPクライアント側のモデルに任せます。フィード由来の抜粋は、AIが生成した要約とは明確に区別されます。

## 収集ソース

| ID | ソース | 区分 | 主な対象 |
| --- | --- | --- | --- |
| `android-developers-blog` | [Android Developers Blog](https://developer.android.com/blog/) | 公式 | Android Platform、Google Play、開発ツール |
| `androidx-release-notes` | [AndroidX Release Notes](https://developer.android.com/jetpack/androidx/versions/all-channel) | 公式 | AndroidX、Jetpack、Compose |
| `kotlin-blog` | [Kotlin Blog](https://blog.jetbrains.com/kotlin/) | 公式 | Kotlin、KMP、CMP、KotlinConf、Wasm |
| `proandroiddev` | [ProAndroidDev](https://proandroiddev.com/) | コミュニティ | Android/Kotlinの実践記事 |
| `android-dagashi` | [AndroidDagashi](https://androiddagashi.github.io/) | コミュニティ | Android関連ニュースの週次まとめ |

公式情報とコミュニティ記事は、すべての出力で`official` / `community`として区別されます。

## 主な機能

- RSS 0.x/1.0/2.0、Atom、JSON Feedの正規化
- 直近N日、ソース、トピック、ソース区分によるフィルタ
- Kotlin、KMP、Compose Multiplatform、KotlinConfなどのトピック付与
- URL正規化と重複除去
- RFC 3339 UTCへの日時統一
- ソース単位の部分失敗と取得状況の報告
- JSON Schema付きMCP structured content
- 人間が確認・共有しやすいMarkdown出力
- 5種類のSlack投稿スキーマと、明示指定時だけ使う暫定Block Kit変換
- stdoutとstderrの分離

## 必要環境

- Rust 1.88以上（edition 2024）

## セットアップ

```bash
cargo build
cargo test
```

## CLI

登録ソースを表示します。

```bash
cargo run -- sources
```

直近7日間の記事をJSONで取得します。

```bash
cargo run -- collect --days 7
```

KMP、CMP、KotlinConfに絞ります。複数の`--topic`はOR条件です。

```bash
cargo run -- collect \
  --days 30 \
  --topic kmp \
  --topic cmp \
  --topic kotlinconf
```

公式情報だけをMarkdownで出力します。

```bash
cargo run -- collect \
  --days 7 \
  --source-kind official \
  --format markdown
```

特定ソースだけを選択できます。

```bash
cargo run -- collect \
  --source kotlin-blog \
  --source proandroiddev \
  --days 14
```

主なオプションは次のとおりです。

| オプション | 既定値 | 説明 |
| --- | ---: | --- |
| `--days` | `7` | 取得期間。1〜365日 |
| `--source` | すべて | ソースID。複数指定可能 |
| `--topic` | すべて | トピック。複数指定可能 |
| `--source-kind` | すべて | `official`または`community` |
| `--limit` | `100` | 最大件数。1〜500件 |
| `--include-undated` | 無効 | 日付不明の記事も含める |
| `--format` | `json` | `json`または`markdown` |
| `--compact` | 無効 | JSONを1行で出力 |

利用できるトピックは次のとおりです。

```text
android, android_platform, androidx, jetpack, compose, kotlin,
kmp, cmp, kotlin_conf, gradle, agp, tools, community
```

`kotlinconf`、`kotlin-multiplatform`、`compose-multiplatform`などの別名もCLIで受け付けます。

トピックごとの推奨収集日時は[トピック別の収集日時](docs/collection-schedule.md)を参照してください。この表は確認作業の運用案であり、自動実行の設定ではありません。

### Slack投稿

Codexが生成する投稿タイプ、必須項目、入力ヒントをJSONで確認できます。

```bash
cargo run -- slack-forms
```

`format-slack`は暫定機能です。通常の投稿生成では使用せず、Block Kit出力、Slackペイロード、またはプレビュー・テストを明示的に求められた場合だけ使用します。

```bash
cargo run -- format-slack --input post.json
```

標準入力を使う場合は、`cargo run -- format-slack < post.json`と実行します。投稿タイプごとの仕様とJSON例は[Slack投稿仕様](docs/slack-posts.md)を参照してください。

Slack上のDroid部ワークフローの入力欄、`#club_droid`への投稿手順、本文テンプレートは[Droid部のSlack投稿ワークフロー](docs/droid-slack-workflows.md)に記録しています。`slack-forms`の出力にも各タイプの`messageTemplate`が含まれます。

## JSON出力

トップレベルをオブジェクトにしているため、記事以外に検索条件、鮮度、部分失敗、件数制限を伝えられます。

```json
{
  "schemaVersion": "1.0",
  "generatedAt": "2026-09-16T03:00:00Z",
  "query": {
    "days": 7,
    "since": "2026-09-09T03:00:00Z",
    "until": "2026-09-16T03:00:00Z",
    "sourceIds": ["kotlin-blog"],
    "topics": ["kmp", "cmp"],
    "sourceKinds": ["official"],
    "limit": 100,
    "includeUndated": false
  },
  "articles": [
    {
      "id": "sha256:...",
      "title": "Article title",
      "url": "https://example.com/article",
      "publishedAt": "2026-09-15T00:00:00Z",
      "updatedAt": null,
      "retrievedAt": "2026-09-16T03:00:00Z",
      "excerpt": "Feed-provided excerpt",
      "language": "en",
      "authors": ["Author"],
      "source": {
        "id": "kotlin-blog",
        "name": "Kotlin Blog",
        "kind": "official"
      },
      "topics": ["kotlin", "kmp"]
    }
  ],
  "sourceReports": [],
  "sourceErrors": [],
  "stats": {
    "fetchedEntries": 12,
    "matchedBeforeLimit": 4,
    "returnedArticles": 4,
    "duplicatesRemoved": 0,
    "skippedUndated": 0,
    "skippedInvalid": 0
  },
  "truncated": false
}
```

`publishedAt`または`updatedAt`がない記事は、既定では期間判定ができないため除外されます。`--include-undated`で含められます。

## MCPサーバー

stdio transportで起動します。

```bash
cargo run -- serve
```

リリースビルドをMCPクライアントへ登録する場合の実行コマンドは次の形です。

```text
/absolute/path/to/droid-mcp/target/release/droid-mcp serve
```

提供するツールは4つです。

### `list_sources`

登録済みソース、フィードURL、公式/コミュニティ区分、既定トピックを返します。

### `get_recent_articles`

次の入力を受け取り、CLIと同じ`ArticleCollection`を返します。

```json
{
  "days": 7,
  "sourceIds": [],
  "topics": ["kotlin", "kmp", "cmp", "kotlin_conf"],
  "sourceKinds": ["official", "community"],
  "limit": 100,
  "includeUndated": false
}
```

すべてのツールは`outputSchema`を公開し、結果を`structuredContent`と互換用のJSONテキストの両方で返します。ツールには読み取り専用のannotationを設定しています。

### `list_slack_post_types`

News、Android OS情報、Compose、Library、質問・相談の5種類について、必須項目、入力形式、日本語のヒント、Slack本文テンプレートを返します。

### `format_slack_post`

Codexが生成した構造化投稿を検証し、`text`と`blocks`を含むBlock Kitペイロードへ変換する暫定ツールです。通常の投稿生成では呼び出さず、ユーザーがBlock Kit出力、Slackペイロード、またはプレビュー・テストを明示的に依頼した場合だけ使用します。このツールはSlackへ投稿しません。

## CodexからSlack投稿を作る流れ

MCPサーバーはクライアントへ、次の順序で処理するよう指示します。

1. `get_recent_articles`で最新情報を取得する
2. Codexが適切な投稿タイプを選ぶ
3. 記事の根拠に基づいて各項目を埋める
4. Slack画面で確認した本文テンプレートに沿って投稿下書きを作る
5. `post/<トピック>/YYYY-MM-DD-<内容を表す名前>.md`にMarkdownファイルとして保存し、パスを返す

分類、投稿文の作成、ファイル出力はCodexの担当です。RustはXML/JSONフィードを収集・正規化し、投稿タイプの定義を公開します。Block Kitが明示的に要求された場合だけ、追加で`format_slack_post`を使って必須項目、URL、文字数を検証し、Rustで同じ本文構成のBlock Kitへ整形します。

Newsの`summary`は記事の内容、`comment`はCodexによる提案コメントとして分離します。Codexのコメントをユーザー本人の発言として扱わず、未実施の検証や対応結果も生成しません。`excerpt`は情報源が配信した抜粋であり、そのままAI要約として表示しません。

CLIの`--format markdown`は、AIによる判断を加えずに事実データと原文抜粋だけを整形します。

## 構成

```text
src/
├── collector.rs   # HTTP取得、並列処理、期間フィルタ、部分失敗
├── feed.rs        # RSS/Atom解析、Article変換、URL正規化、トピック付与
├── lib.rs         # 共通ライブラリ
├── main.rs        # CLIとstdio MCPの起動
├── markdown.rs    # 共有用の事実ベースMarkdown
├── mcp.rs         # 薄いMCPインターフェース
├── model.rs       # 入出力モデルとJSON Schema
├── slack.rs       # Slack投稿仕様、検証、Block Kit整形
└── sources.rs     # 組み込みソースレジストリ
```

CLIとMCPは同じ`Collector`を利用します。MCP固有型はフィード解析やドメイン処理へ入りません。

## 安全性と制限

- フィード取得はHTTPSの組み込みURLだけに限定しています。
- 接続・リクエストタイムアウト、リダイレクト上限、4 MiBのレスポンス上限があります。
- 1ソースの失敗で、他ソースの成功結果を破棄しません。
- stdoutにはCLIの結果またはMCPメッセージだけを出し、ログとエラーはstderrへ出します。
- RSSに残っている範囲より古い記事を遡れる保証はありません。`sourceReports.oldestAvailableAt`で取得できた範囲を確認できます。
- トピックはフィードのカテゴリとキーワードによる決定的な分類です。意味的な分類や重要度判定はMCPクライアント側で行います。
- 任意URLの本文取得はSSRFと著作権上のリスクがあるため、現段階では公開していません。
- Slack整形はペイロード生成までで、API送信、トークン、Webhook URLは扱いません。

## 検証

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- collect --days 7 --limit 5
```
