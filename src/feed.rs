use std::{collections::BTreeSet, io::Cursor};

use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use sha2::{Digest, Sha256};
use url::Url;

use crate::model::{Article, ArticleSource, SourceDefinition, Topic};

const MAX_EXCERPT_CHARS: usize = 1_200;

#[derive(Debug)]
pub struct ParsedFeed {
    pub articles: Vec<Article>,
    pub fetched_entries: usize,
    pub skipped_invalid: usize,
}

pub fn parse_source_feed(
    source: &SourceDefinition,
    bytes: &[u8],
    retrieved_at: DateTime<Utc>,
) -> Result<ParsedFeed, feed_rs::parser::ParseFeedError> {
    let feed = feed_rs::parser::parse(bytes)?;
    let fetched_entries = feed.entries.len();
    let mut skipped_invalid = 0;
    let mut articles = Vec::with_capacity(fetched_entries);

    for entry in feed.entries {
        match entry_to_article(source, entry, retrieved_at) {
            Some(article) => articles.push(article),
            None => skipped_invalid += 1,
        }
    }

    Ok(ParsedFeed {
        articles,
        fetched_entries,
        skipped_invalid,
    })
}

fn entry_to_article(
    source: &SourceDefinition,
    entry: Entry,
    retrieved_at: DateTime<Utc>,
) -> Option<Article> {
    let title = entry.title.as_ref().map(|text| plain_text(&text.content))?;
    if title.is_empty() {
        return None;
    }

    let raw_url = preferred_link(&entry)?;
    let url = normalize_url(raw_url)?;
    let excerpt = entry
        .summary
        .as_ref()
        .map(|summary| summary.content.as_str())
        .or_else(|| {
            entry
                .content
                .as_ref()
                .and_then(|content| content.body.as_deref())
        })
        .map(plain_text)
        .filter(|text| !text.is_empty())
        .map(|text| truncate_chars(&text, MAX_EXCERPT_CHARS));

    let category_text = entry
        .categories
        .iter()
        .flat_map(|category| [Some(category.term.as_str()), category.label.as_deref()])
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let topics = classify_topics(
        source,
        &format!(
            "{} {} {}",
            title,
            excerpt.as_deref().unwrap_or_default(),
            category_text
        ),
    );

    let authors = entry
        .authors
        .iter()
        .map(|author| author.name.trim())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Some(Article {
        id: article_id(&url, &title),
        title,
        url,
        published_at: entry.published,
        updated_at: entry.updated,
        retrieved_at,
        excerpt,
        language: entry.language,
        authors,
        source: ArticleSource::from(source),
        topics,
    })
}

fn preferred_link(entry: &Entry) -> Option<&str> {
    entry
        .links
        .iter()
        .find(|link| link.rel.as_deref().is_none_or(|rel| rel == "alternate"))
        .or_else(|| entry.links.first())
        .map(|link| link.href.as_str())
}

pub fn normalize_url(raw: &str) -> Option<String> {
    let mut url = Url::parse(raw.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    url.set_fragment(None);

    let is_medium = url.host_str().is_some_and(|host| {
        host == "medium.com" || host.ends_with(".medium.com") || host == "proandroiddev.com"
    });
    let mut retained_pairs = url
        .query_pairs()
        .filter(|(key, _)| !(is_tracking_parameter(key) || is_medium && key == "source"))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    retained_pairs.sort();

    if retained_pairs.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(retained_pairs);
    }

    Some(url.into())
}

fn is_tracking_parameter(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.starts_with("utm_")
        || matches!(
            key.as_str(),
            "gclid" | "dclid" | "fbclid" | "mc_cid" | "mc_eid" | "ref_src"
        )
}

fn article_id(url: &str, title: &str) -> String {
    let identity = format!("{}\n{}", url, title.trim().to_ascii_lowercase());
    format!("sha256:{:x}", Sha256::digest(identity.as_bytes()))
}

fn plain_text(value: &str) -> String {
    let rendered = html2text::from_read_with_decorator(
        Cursor::new(value.as_bytes()),
        120,
        html2text::render::TrivialDecorator::new(),
    )
    .unwrap_or_else(|_| value.to_owned());
    rendered.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_owned();
    }

    let mut truncated = value
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

pub fn classify_topics(source: &SourceDefinition, text: &str) -> Vec<Topic> {
    let haystack = text.to_ascii_lowercase();
    let mut topics = source
        .default_topics
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    if contains_any(&haystack, &["kotlinconf", "kotlin conf"]) {
        topics.insert(Topic::KotlinConf);
        topics.insert(Topic::Kotlin);
    }

    let mentions_multiplatform =
        contains_any(&haystack, &["kotlin multiplatform", "multiplatform"])
            || contains_token(&haystack, "kmp");
    let mentions_compose = contains_any(&haystack, &["compose", "jetpack compose"]);

    if mentions_multiplatform {
        topics.insert(Topic::Kmp);
        topics.insert(Topic::Kotlin);
    }
    if haystack.contains("compose multiplatform")
        || contains_token(&haystack, "cmp")
        || (mentions_compose && mentions_multiplatform)
    {
        topics.insert(Topic::Cmp);
        topics.insert(Topic::Compose);
        topics.insert(Topic::Kmp);
    } else if mentions_compose {
        topics.insert(Topic::Compose);
    }
    if contains_any(&haystack, &["kotlin", "coroutine", "ktor"]) {
        topics.insert(Topic::Kotlin);
    }
    if contains_any(&haystack, &["androidx", "jetpack"]) {
        topics.insert(Topic::Androidx);
        topics.insert(Topic::Jetpack);
    }
    if contains_any(
        &haystack,
        &[
            "behavior change",
            "behaviour change",
            "target sdk",
            "targetsdk",
            "api level",
            "android platform",
            "android 16",
            "android 17",
        ],
    ) {
        topics.insert(Topic::AndroidPlatform);
        topics.insert(Topic::Android);
    }
    if haystack.contains("gradle") {
        topics.insert(Topic::Gradle);
    }
    if haystack.contains("android gradle plugin") || contains_token(&haystack, "agp") {
        topics.insert(Topic::Agp);
        topics.insert(Topic::Gradle);
    }
    if contains_any(
        &haystack,
        &[
            "android studio",
            "intellij idea",
            "tooling",
            "developer tool",
        ],
    ) {
        topics.insert(Topic::Tools);
    }

    topics.into_iter().collect()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn contains_token(haystack: &str, token: &str) -> bool {
    haystack
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == token)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::sources::built_in_sources;

    #[test]
    fn normalizes_tracking_parameters_and_fragments() {
        let normalized =
            normalize_url("https://example.com/post?utm_source=rss&b=2&a=1#heading").unwrap();

        assert_eq!(normalized, "https://example.com/post?a=1&b=2");
    }

    #[test]
    fn removes_medium_source_tracking_parameter() {
        let normalized =
            normalize_url("https://proandroiddev.com/article-123?source=rss----c72404660798---4")
                .unwrap();

        assert_eq!(normalized, "https://proandroiddev.com/article-123");
    }

    #[test]
    fn rejects_non_http_urls() {
        assert_eq!(normalize_url("file:///etc/passwd"), None);
        assert_eq!(normalize_url("javascript:alert(1)"), None);
    }

    #[test]
    fn identifies_kmp_cmp_and_kotlinconf_topics() {
        let source = built_in_sources()
            .into_iter()
            .find(|source| source.id == "kotlin-blog")
            .unwrap();
        let topics = classify_topics(
            &source,
            "KotlinConf: Compose Multiplatform and Kotlin language updates",
        );

        assert!(topics.contains(&Topic::Kotlin));
        assert!(topics.contains(&Topic::Kmp));
        assert!(topics.contains(&Topic::Cmp));
        assert!(topics.contains(&Topic::Compose));
        assert!(topics.contains(&Topic::KotlinConf));
    }

    #[test]
    fn parses_rss_into_normalized_article() {
        let source = built_in_sources()
            .into_iter()
            .find(|source| source.id == "proandroiddev")
            .unwrap();
        let retrieved_at = Utc.with_ymd_and_hms(2026, 9, 16, 3, 0, 0).unwrap();
        let rss = br#"<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
              <channel>
                <title>Example</title>
                <link>https://example.com</link>
                <description>Example feed</description>
                <item>
                  <title>Compose Multiplatform update</title>
                  <link>https://example.com/post?utm_source=rss</link>
                  <description><![CDATA[<p>A useful <strong>KMP</strong> update.</p>]]></description>
                  <pubDate>Tue, 15 Sep 2026 00:00:00 +0000</pubDate>
                  <author>author@example.com (Jane Doe)</author>
                </item>
              </channel>
            </rss>"#;

        let parsed = parse_source_feed(&source, rss, retrieved_at).unwrap();

        assert_eq!(parsed.fetched_entries, 1);
        assert_eq!(parsed.skipped_invalid, 0);
        assert_eq!(parsed.articles.len(), 1);
        let article = &parsed.articles[0];
        assert_eq!(article.url, "https://example.com/post");
        assert_eq!(article.excerpt.as_deref(), Some("A useful KMP update."));
        assert!(article.topics.contains(&Topic::Cmp));
        assert!(article.topics.contains(&Topic::Kmp));
    }
}
