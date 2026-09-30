# トピック別の収集日時（運用案）

日時は日本時間（JST）です。以下は情報を確認する**推奨日時**であり、記事の公開日時や、このアプリによる自動実行の設定ではありません。各曜日の**11:00ごろ**を目安にします。日曜夜更新のAndroidDagashiを翌朝に確認でき、ほかのソースも前日までの更新をまとめて確認しやすいためです。公開曜日の目安を確認できたのは、AndroidDagashiの「おおよそ毎週日曜夜」だけです。ほかの曜日と時刻は週内で確認作業を分けるための案です。

| トピック（`--topic`の値） | 推奨収集日時（JST） | 確認する内容 |
| --- | --- | --- |
| `android` | 毎週月曜 11:00 | Android関連の新着全般 |
| `android_platform` | 毎週月曜 11:00 | OS、API、動作変更 |
| `androidx` | 毎週木曜 11:00 | AndroidXのリリース情報 |
| `jetpack` | 毎週木曜 11:00 | Jetpackライブラリの更新 |
| `compose` | 毎週木曜 11:00 | Jetpack Compose関連の更新 |
| `kotlin` | 毎週火曜 11:00 | Kotlinの言語・エコシステム |
| `kmp` | 毎週火曜 11:00 | Kotlin Multiplatform |
| `cmp` | 毎週火曜 11:00 | Compose Multiplatform |
| `kotlin_conf` | 毎週火曜 11:00 | KotlinConf関連の発表・記事 |
| `gradle` | 毎週金曜 11:00 | Gradle関連の更新 |
| `agp` | 毎週金曜 11:00 | Android Gradle Plugin関連の更新 |
| `tools` | 毎週金曜 11:00 | Android Studioなどの開発ツール |
| `community` | 毎週月曜 11:00 | コミュニティ記事とAndroidDagashi |

AndroidDagashiは`community`というトピックそのものではなく、`android-dagashi`という**ソース**です。[公式サイト](https://androiddagashi.github.io/)は「おおよそ毎週日曜夜の更新」と案内しています。月曜に最新号を確認し、まだ出ていなければ後日再確認します。AndroidDagashiだけを取得する場合は`--source android-dagashi`を使います。

各曜日のトピックは、例えば次のようにまとめて取得できます。複数の`--topic`はOR条件なので、結果をトピック別に扱う際は各記事の`topics`を確認します。

```bash
# 月曜: Android全般、OS、コミュニティ
cargo run -- collect --days 8 --topic android --topic android_platform --topic community

# 火曜: Kotlin関連
cargo run -- collect --days 8 --topic kotlin --topic kmp --topic cmp --topic kotlin_conf

# 木曜: AndroidX、Jetpack、Compose
cargo run -- collect --days 8 --topic androidx --topic jetpack --topic compose

# 金曜: ビルドと開発ツール
cargo run -- collect --days 8 --topic gradle --topic agp --topic tools
```

`--days 8`は前週と少し重なる取得範囲です。同じ記事を再度扱わないよう、記事のURLまたは`id`で既出を確認してください。複数曜日の結果に同じ記事が入っても投稿は1本とし、公開日時より前の曜日を避けて、内容に最も合う曜日・トピックに割り当てます。まとめ記事と個別記事で同じニュースを扱う場合は、個別記事を優先し、まとめ記事では未紹介の話題だけを取り上げます。収集結果が空の場合は、対象期間内に該当記事がなかったのか、`sourceErrors`や`sourceReports`に取得失敗・フィード掲載範囲の不足がないかを確認します。緊急のリリースやイベント情報は、この曜日を待たずに確認します。

MCPの`get_recent_articles`でも`topics`配列と`days`で同じ条件を指定できます。Rustアプリ自身には定期実行機能はなく、この表だけで自動実行は始まりません。

## 実行結果のまとめ方

最初に実行日時、対象トピック、取得期間、該当件数、取得エラーの有無を示します。各項目は「タイトル・日付・ソース・URL」に続けて、内容を一文で概要化します。概要は技術や変更点を主語にし、「Compose 1.12.1が公開されました」「tvOS向けのターゲットとsource setを分け、共通コードを共有する構成です」のように書きます。「記事が〜する」や「記事で〜が紹介されています」の反復は避けます。「詳細はURLを参照」など、リンクがあれば明らかな案内も書きません。

本文まで確認できなかった記事は、フィードの抜粋だけで原因や結論を補わず、確認できた範囲を短く明示します。
