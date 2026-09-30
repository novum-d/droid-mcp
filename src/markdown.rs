use std::fmt::Write;

use crate::model::ArticleCollection;

pub fn render_markdown(collection: &ArticleCollection) -> String {
    let mut output = String::new();
    writeln!(output, "# Android / Kotlin 最新情報").unwrap();
    writeln!(output).unwrap();
    writeln!(
        output,
        "- 対象期間: {} 〜 {}",
        collection.query.since.to_rfc3339(),
        collection.query.until.to_rfc3339()
    )
    .unwrap();
    writeln!(
        output,
        "- 生成日時: {}",
        collection.generated_at.to_rfc3339()
    )
    .unwrap();
    writeln!(
        output,
        "- 取得件数: {}件",
        collection.stats.returned_articles
    )
    .unwrap();
    writeln!(
        output,
        "- 注記: 本文はフィード由来の抜粋であり、AI要約ではありません。"
    )
    .unwrap();

    if !collection.source_errors.is_empty() {
        writeln!(output).unwrap();
        writeln!(output, "## 取得できなかったソース").unwrap();
        writeln!(output).unwrap();
        for error in &collection.source_errors {
            writeln!(
                output,
                "- `{}`: {:?} — {}",
                error.source_id, error.code, error.message
            )
            .unwrap();
        }
    }

    for article in &collection.articles {
        writeln!(output).unwrap();
        writeln!(output, "## {}", escape_heading(&article.title)).unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "- 情報源: {}（{}）",
            article.source.name, article.source.kind
        )
        .unwrap();
        writeln!(
            output,
            "- 公開日時: {}",
            article
                .published_at
                .or(article.updated_at)
                .map(|date| date.to_rfc3339())
                .unwrap_or_else(|| "不明".to_owned())
        )
        .unwrap();
        writeln!(
            output,
            "- トピック: {}",
            article
                .topics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        if !article.authors.is_empty() {
            writeln!(output, "- 著者: {}", article.authors.join(", ")).unwrap();
        }
        if let Some(excerpt) = &article.excerpt {
            writeln!(output).unwrap();
            writeln!(output, "{}", excerpt).unwrap();
        }
        writeln!(output).unwrap();
        writeln!(output, "[原文を読む]({})", article.url).unwrap();
    }

    output
}

fn escape_heading(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::model::{
        Article, ArticleSource, CollectRequest, CollectionStats, QueryEcho, SourceKind, Topic,
    };

    #[test]
    fn markdown_distinguishes_excerpt_from_ai_summary() {
        let now = Utc.with_ymd_and_hms(2026, 9, 16, 3, 0, 0).unwrap();
        let collection = ArticleCollection {
            schema_version: "1.0".into(),
            generated_at: now,
            query: QueryEcho {
                days: 7,
                since: now - chrono::TimeDelta::days(7),
                until: now,
                source_ids: vec!["kotlin-blog".into()],
                topics: CollectRequest::default().topics,
                source_kinds: Vec::new(),
                limit: 100,
                include_undated: false,
            },
            articles: vec![Article {
                id: "id".into(),
                title: "Kotlin update".into(),
                url: "https://example.com/article".into(),
                published_at: Some(now),
                updated_at: Some(now),
                retrieved_at: now,
                excerpt: Some("Feed excerpt".into()),
                language: Some("en".into()),
                authors: vec!["Jane Doe".into()],
                source: ArticleSource {
                    id: "kotlin-blog".into(),
                    name: "Kotlin Blog".into(),
                    kind: SourceKind::Official,
                },
                topics: vec![Topic::Kotlin],
            }],
            source_reports: Vec::new(),
            source_errors: Vec::new(),
            stats: CollectionStats {
                returned_articles: 1,
                ..CollectionStats::default()
            },
            truncated: false,
        };

        let rendered = render_markdown(&collection);
        assert!(rendered.contains("AI要約ではありません"));
        assert!(rendered.contains("Kotlin Blog（official）"));
        assert!(rendered.contains("[原文を読む](https://example.com/article)"));
    }
}
