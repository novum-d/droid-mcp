# Slack投稿仕様

Droid MCPはRSS/AtomのXMLとJSON Feedを取得・正規化します。収集結果やユーザーが渡したJSON・XMLの内容から投稿タイプを選び、投稿文を作成してファイルに残すのはCodexの担当です。Slackへの送信や認証情報の管理は行いません。

Block Kit整形は暫定機能です。通常の投稿生成では使用せず、ユーザーがBlock Kit出力、Slackペイロード、またはプレビュー・テストを明示的に依頼した場合だけ使用します。

```text
get_recent_articles
  → Codexが投稿タイプを選ぶ
  → Codexが各項目を根拠に基づいて埋める
  → Slack本文テンプレートで投稿下書きを作る
  → post/<トピック>/YYYY-MM-DD-<内容を表す名前>.md にMarkdownで保存する

明示的にBlock Kitが必要な場合のみ:
  Codexが作った各項目の構造化JSON
    → format_slack_post
    → Rustが検証して同じ本文構成のBlock Kitへ変換する
```

本文の見出しと項目順は[Droid部のSlack投稿ワークフロー](droid-slack-workflows.md)に記録しています。`list_slack_post_types`も各投稿タイプの`messageTemplate`を返します。根拠がない任意項目は見出しごと省きます。

## 生成時の原則

- 記事に書かれている内容とCodexによる見解を分離します。
- Newsの`summary`は記事内容、`comment`はCodexによる提案コメントです。
- Codexのコメントをユーザー本人の発言として扱いません。
- 実施していない検証や対応を「対応・調査結果」に記載しません。未確認の場合は、その旨を明示します。
- Android OSの挙動を断定する場合は、Android Developers、AOSP、Issue Trackerなどの一次情報を優先します。
- フィードの抜粋だけでは根拠が足りない項目は、空欄にするか確認が必要であることを示します。

## 投稿タイプ

### 📰 News投稿

| JSONキー | 項目 | 必須 | ヒント |
| --- | --- | :---: | --- |
| `title` | タイトル | ○ | `例：Android 17 Beta 2 がリリース` |
| `url` | URL | ○ | `共有する記事・公式ドキュメントなどのURL` |
| `summary` | 概要 |  | `記事の内容を1〜3行程度で簡単にまとめてください` |
| `comment` | コメント |  | `気になった点や、共有したいと思った理由など` |

`summary`（概要）は**記事そのものの内容**、`comment`（コメント）は**投稿者自身の視点**として別々に生成します。記事の事実と投稿者の意見を混ぜません。

### 🤖 Android OS情報投稿

| JSONキー | 項目 | 必須 | ヒント |
| --- | --- | :---: | --- |
| `title` | タイトル | ○ | `例：Android 17で大画面のOrientation制限が変更` |
| `targetOs` | 対象OS |  | `例：Android 17 / API 37` |
| `targetSdk` | 対象targetSdk |  | `例：全アプリ / targetSdk 37以上` |
| `changeDetails` | 変更内容 | ○ | `従来の挙動から何が変わるのかを記載してください` |
| `appImpact` | アプリへの影響 |  | `既存アプリで影響を受ける画面・機能・条件など` |
| `responseFindings` | 対応・調査結果 |  | `必要な対応、検証結果、現時点での判断など` |
| `referenceUrl` | 参考URL | ○ | `Android Developers / AOSP / Issue Trackerなどの一次情報を推奨` |

`targetSdk`のヒントは、全アプリに適用される変更と、特定のtargetSdkを対象にする変更の区別にも使います。

### 🧩 Compose投稿

| JSONキー | 項目 | 必須 | ヒント |
| --- | --- | :---: | --- |
| `title` | タイトル | ○ | `例：Modifier.Nodeを使った独自Modifierの実装` |
| `summary` | 概要 | ○ | `何ができるのか、何が変わったのかを簡単に記載してください` |
| `sampleNotes` | サンプル・補足 |  | `コード例、利用時の注意点、実際に試した感想など` |
| `referenceUrl` | 参考URL |  | `公式ドキュメント、Release Notes、記事、リポジトリなど` |

### 📦 Library投稿

| JSONキー | 項目 | 必須 | ヒント |
| --- | --- | :---: | --- |
| `libraryName` | ライブラリ名 | ○ | `例：Coil / Retrofit / kotlinx.coroutines` |
| `title` | タイトル | ○ | `例：Coil 3.4.0がリリース` |
| `version` | バージョン |  | `例：3.4.0` |
| `summary` | 概要 | ○ | `主な変更点、導入メリット、Breaking Changeなど` |
| `referenceUrl` | 参考URL |  | `GitHub Releases、公式ドキュメントなど` |

### ❓ 質問・相談

| JSONキー | 項目 | 必須 | ヒント |
| --- | --- | :---: | --- |
| `title` | タイトル | ○ | `例：Reader ModeとForeground Dispatchの使い分けについて` |
| `question` | 質問・困っていること | ○ | `知りたいことや、現在困っていることを記載してください` |
| `tried` | 試したこと |  | `調べたこと、試した実装、確認済みの内容など` |
| `environment` | 環境 |  | `例：Android 16 / Pixel 8 / Kotlin 2.x / Compose` |
| `referencesOrCode` | 参考URL・コードなど |  | `関連ドキュメント、Issue、コード、エラーログなど` |

「試したこと」は、質問のハードルを下げるため任意です。

## JSONからBlock Kitへの変換（暫定・明示指定時のみ）

以下の機能は通常の生成フローでは使いません。

CLIでは、投稿内容のJSONを直接渡します。

```json
{
  "postType": "news",
  "title": "Android 17 Beta 2 がリリース",
  "url": "https://developer.android.com/about/versions/17/",
  "summary": "Android 17 Beta 2が公開され、APIと動作変更の検証版が更新されました。",
  "comment": "大画面対応と既存アプリへの影響を先に確認しておきたい内容です。"
}
```

```bash
cargo run -- format-slack --input post.json
```

標準入力も利用できます。

```bash
cargo run -- format-slack < post.json
```

結果の`message`オブジェクトが送信可能なSlackメッセージです。

```json
{
  "schemaVersion": "1.0",
  "postType": "news",
  "message": {
    "text": "【📰 News投稿】\nAndroid 17 Beta 2 がリリース\n────────────────────\nAndroid 17 Beta 2が公開され、APIと動作変更の検証版が更新されました。\n💬 ── コメント ──\n大画面対応と既存アプリへの影響を先に確認しておきたい内容です。\n🔗 ── 参考URL ──\nhttps://developer.android.com/about/versions/17/",
    "blocks": [
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*【📰 News投稿】*\n*Android 17 Beta 2 がリリース*"
        }
      },
      {
        "type": "divider"
      },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "Android 17 Beta 2が公開され、APIと動作変更の検証版が更新されました。"
        }
      },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*💬 ── コメント ──*\n大画面対応と既存アプリへの影響を先に確認しておきたい内容です。"
        }
      },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*🔗 ── 参考URL ──*\n<https://developer.android.com/about/versions/17/|https://developer.android.com/about/versions/17/>"
        }
      }
    ]
  }
}
```

MCPの`format_slack_post`では、MCP仕様上のルート入力をオブジェクトにするため`post`で包みます。

```json
{
  "post": {
    "postType": "news",
    "title": "Android 17 Beta 2 がリリース",
    "url": "https://developer.android.com/about/versions/17/",
    "summary": "Android 17 Beta 2が公開されました。",
    "comment": "既存アプリへの影響を確認したい内容です。"
  }
}
```

## Rust側の責務

以下は`format_slack_post`が明示的に呼び出された場合の責務です。

- 投稿タイプごとの必須項目検証
- HTTP/HTTPS URLの検証
- Slack制御文字`&`、`<`、`>`のエスケープ
- `@channel`相当の意図しないメンション構文の無効化
- Block Kitの`section`上限を考慮した文字数制限
- 通知とスクリーンリーダー向けのトップレベル`text`生成
- 絵文字、見出し、区切り、リンクの統一

Block Kitは、Slackメッセージを`section`や`divider`などのJSONブロックで構成する仕組みです。仕様は[Slack Block Kit](https://docs.slack.dev/block-kit/)と[メッセージ本文のフォーマット](https://docs.slack.dev/messaging/formatting-message-text/)を参照してください。

## 対象外

- Slack APIやIncoming Webhookへの送信
- Slackトークン、Webhook URL、チャンネルIDの保持
- Codexが生成した投稿の自動承認
- Slack Workflowフォームの自動作成

送信処理を追加する場合も、`SlackFormattedPost.message`を明示的な確認後に渡す構成を推奨します。
