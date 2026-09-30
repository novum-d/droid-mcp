use std::{collections::BTreeMap, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use futures_util::{StreamExt, stream};
use reqwest::{Client, StatusCode, header};
use thiserror::Error;
use tokio::time::sleep;

use crate::{
    feed::{ParsedFeed, parse_source_feed},
    model::{
        Article, ArticleCollection, CollectRequest, CollectionStats, QueryEcho, SCHEMA_VERSION,
        SourceDefinition, SourceError, SourceErrorCode, SourceList, SourceReport,
    },
    sources::built_in_sources,
};

const MAX_FEED_BYTES: usize = 4 * 1024 * 1024;
const MAX_CONCURRENT_REQUESTS: usize = 4;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ATTEMPTS: usize = 2;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("failed to build the HTTP client: {0}")]
    ClientBuild(#[source] reqwest::Error),
    #[error("days must be between 1 and 365 (received {0})")]
    InvalidDays(u32),
    #[error("limit must be between 1 and 500 (received {0})")]
    InvalidLimit(usize),
    #[error("unknown source ID(s): {0}")]
    UnknownSources(String),
    #[error("the selected source and source-kind filters match no sources")]
    NoSourcesSelected,
}

#[derive(Clone)]
pub struct Collector {
    client: Client,
    sources: Vec<SourceDefinition>,
}

impl std::fmt::Debug for Collector {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Collector")
            .field("sources", &self.sources)
            .finish_non_exhaustive()
    }
}

impl Collector {
    pub fn new() -> Result<Self, CollectorError> {
        Self::with_sources(built_in_sources())
    }

    pub fn with_sources(sources: Vec<SourceDefinition>) -> Result<Self, CollectorError> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent(concat!("droid-mcp/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(CollectorError::ClientBuild)?;

        Ok(Self { client, sources })
    }

    pub fn list_sources(&self) -> SourceList {
        SourceList {
            schema_version: SCHEMA_VERSION.to_owned(),
            sources: self.sources.clone(),
        }
    }

    pub async fn collect(
        &self,
        request: CollectRequest,
    ) -> Result<ArticleCollection, CollectorError> {
        self.collect_at(request, Utc::now()).await
    }

    async fn collect_at(
        &self,
        request: CollectRequest,
        generated_at: DateTime<Utc>,
    ) -> Result<ArticleCollection, CollectorError> {
        validate_request(&request)?;
        let selected_sources = self.select_sources(&request)?;
        let since = generated_at - TimeDelta::days(i64::from(request.days));

        let results = stream::iter(selected_sources.iter().cloned().map(|source| async move {
            let result = self.fetch_and_parse(&source, generated_at).await;
            (source, result)
        }))
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

        let mut source_reports = Vec::new();
        let mut source_errors = Vec::new();
        let mut matching_articles = Vec::new();

        for (source, result) in results {
            match result {
                Ok(parsed) => {
                    let (report, mut articles) =
                        filter_source_articles(&source, parsed, &request, since, generated_at);
                    source_reports.push(report);
                    matching_articles.append(&mut articles);
                }
                Err(error) => source_errors.push(error),
            }
        }

        source_reports.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        source_errors.sort_by(|left, right| left.source_id.cmp(&right.source_id));

        let fetched_entries = source_reports
            .iter()
            .map(|report| report.fetched_entries)
            .sum();
        let skipped_undated = source_reports
            .iter()
            .map(|report| report.skipped_undated)
            .sum();
        let skipped_invalid = source_reports
            .iter()
            .map(|report| report.skipped_invalid)
            .sum();

        let mut by_id = BTreeMap::<String, Article>::new();
        let mut duplicates_removed = 0;
        for article in matching_articles {
            match by_id.entry(article.id.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(article);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    duplicates_removed += 1;
                    if article.source.kind < entry.get().source.kind {
                        entry.insert(article);
                    }
                }
            }
        }

        let mut articles = by_id.into_values().collect::<Vec<_>>();
        articles.sort_by(|left, right| {
            right
                .effective_date()
                .cmp(&left.effective_date())
                .then_with(|| left.id.cmp(&right.id))
        });

        let matched_before_limit = articles.len();
        let truncated = matched_before_limit > request.limit;
        articles.truncate(request.limit);
        let returned_articles = articles.len();
        let selected_source_ids = selected_sources
            .iter()
            .map(|source| source.id.clone())
            .collect();

        Ok(ArticleCollection {
            schema_version: SCHEMA_VERSION.to_owned(),
            generated_at,
            query: QueryEcho {
                days: request.days,
                since,
                until: generated_at,
                source_ids: selected_source_ids,
                topics: request.topics,
                source_kinds: request.source_kinds,
                limit: request.limit,
                include_undated: request.include_undated,
            },
            articles,
            source_reports,
            source_errors,
            stats: CollectionStats {
                fetched_entries,
                matched_before_limit,
                returned_articles,
                duplicates_removed,
                skipped_undated,
                skipped_invalid,
            },
            truncated,
        })
    }

    fn select_sources(
        &self,
        request: &CollectRequest,
    ) -> Result<Vec<SourceDefinition>, CollectorError> {
        if !request.source_ids.is_empty() {
            let mut unknown = request
                .source_ids
                .iter()
                .filter(|requested| !self.sources.iter().any(|source| &source.id == *requested))
                .cloned()
                .collect::<Vec<_>>();
            unknown.sort();
            unknown.dedup();
            if !unknown.is_empty() {
                return Err(CollectorError::UnknownSources(unknown.join(", ")));
            }
        }

        let selected = self
            .sources
            .iter()
            .filter(|source| {
                request.source_ids.is_empty() || request.source_ids.contains(&source.id)
            })
            .filter(|source| {
                request.source_kinds.is_empty() || request.source_kinds.contains(&source.kind)
            })
            .cloned()
            .collect::<Vec<_>>();

        if selected.is_empty() {
            return Err(CollectorError::NoSourcesSelected);
        }
        Ok(selected)
    }

    async fn fetch_and_parse(
        &self,
        source: &SourceDefinition,
        retrieved_at: DateTime<Utc>,
    ) -> Result<ParsedFeed, SourceError> {
        let bytes = self.fetch_feed(source).await?;
        parse_source_feed(source, &bytes, retrieved_at).map_err(|error| SourceError {
            source_id: source.id.clone(),
            code: SourceErrorCode::ParseFailed,
            message: format!("could not parse feed: {error}"),
        })
    }

    async fn fetch_feed(&self, source: &SourceDefinition) -> Result<Vec<u8>, SourceError> {
        let mut last_error = None;

        for attempt in 1..=MAX_ATTEMPTS {
            match self.fetch_feed_once(source).await {
                Ok(bytes) => return Ok(bytes),
                Err(failure) => {
                    let should_retry = failure.retryable && attempt < MAX_ATTEMPTS;
                    last_error = Some(failure.error);
                    if should_retry {
                        sleep(Duration::from_millis(250 * attempt as u64)).await;
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error.expect("at least one fetch attempt is always made"))
    }

    async fn fetch_feed_once(&self, source: &SourceDefinition) -> Result<Vec<u8>, FetchFailure> {
        let response = self
            .client
            .get(&source.feed_url)
            .send()
            .await
            .map_err(|error| FetchFailure::fetch(source, error.to_string(), true))?;

        let status = response.status();
        if !status.is_success() {
            return Err(FetchFailure::fetch(
                source,
                format!("feed returned HTTP {status}"),
                status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error(),
            ));
        }

        if let Some(length) = response.content_length()
            && length > MAX_FEED_BYTES as u64
        {
            return Err(FetchFailure::new(
                SourceError {
                    source_id: source.id.clone(),
                    code: SourceErrorCode::ResponseTooLarge,
                    message: format!(
                        "feed declares {length} bytes; maximum is {MAX_FEED_BYTES} bytes"
                    ),
                },
                false,
            ));
        }

        if let Some(content_type) = response.headers().get(header::CONTENT_TYPE)
            && let Ok(content_type) = content_type.to_str()
            && !is_supported_feed_content_type(content_type)
        {
            return Err(FetchFailure::new(
                SourceError {
                    source_id: source.id.clone(),
                    code: SourceErrorCode::UnsupportedContentType,
                    message: format!("unsupported feed content type '{content_type}'"),
                },
                false,
            ));
        }

        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|error| FetchFailure::fetch(source, error.to_string(), true))?;
            if body.len() + chunk.len() > MAX_FEED_BYTES {
                return Err(FetchFailure::new(
                    SourceError {
                        source_id: source.id.clone(),
                        code: SourceErrorCode::ResponseTooLarge,
                        message: format!("feed exceeded the {MAX_FEED_BYTES}-byte limit"),
                    },
                    false,
                ));
            }
            body.extend_from_slice(&chunk);
        }

        Ok(body)
    }
}

fn validate_request(request: &CollectRequest) -> Result<(), CollectorError> {
    if !(1..=365).contains(&request.days) {
        return Err(CollectorError::InvalidDays(request.days));
    }
    if !(1..=500).contains(&request.limit) {
        return Err(CollectorError::InvalidLimit(request.limit));
    }
    Ok(())
}

fn filter_source_articles(
    source: &SourceDefinition,
    parsed: ParsedFeed,
    request: &CollectRequest,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> (SourceReport, Vec<Article>) {
    let oldest_available_at = parsed
        .articles
        .iter()
        .filter_map(Article::effective_date)
        .min();
    let newest_available_at = parsed
        .articles
        .iter()
        .filter_map(Article::effective_date)
        .max();
    let mut skipped_undated = 0;
    let mut matching = Vec::new();

    for article in parsed.articles {
        let date_matches = match article.effective_date() {
            Some(date) => date >= since && date <= until,
            None if request.include_undated => true,
            None => {
                skipped_undated += 1;
                false
            }
        };
        let topic_matches = request.topics.is_empty()
            || request
                .topics
                .iter()
                .any(|topic| article.topics.contains(topic));

        if date_matches && topic_matches {
            matching.push(article);
        }
    }

    let report = SourceReport {
        source_id: source.id.clone(),
        fetched_entries: parsed.fetched_entries,
        matched_entries: matching.len(),
        skipped_undated,
        skipped_invalid: parsed.skipped_invalid,
        oldest_available_at,
        newest_available_at,
    };

    (report, matching)
}

fn is_supported_feed_content_type(content_type: &str) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    mime.contains("xml")
        || mime.contains("rss")
        || mime.contains("atom")
        || mime == "application/json"
        || mime == "application/feed+json"
        || mime == "text/plain"
        || mime == "application/octet-stream"
}

struct FetchFailure {
    error: SourceError,
    retryable: bool,
}

impl FetchFailure {
    fn new(error: SourceError, retryable: bool) -> Self {
        Self { error, retryable }
    }

    fn fetch(source: &SourceDefinition, message: String, retryable: bool) -> Self {
        Self::new(
            SourceError {
                source_id: source.id.clone(),
                code: SourceErrorCode::FetchFailed,
                message,
            },
            retryable,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ArticleSource, SourceKind, Topic};
    use chrono::TimeZone;

    fn article(
        id: &str,
        date: Option<DateTime<Utc>>,
        source: &SourceDefinition,
        topics: Vec<Topic>,
    ) -> Article {
        Article {
            id: id.into(),
            title: id.into(),
            url: format!("https://example.com/{id}"),
            published_at: date,
            updated_at: date,
            retrieved_at: Utc.with_ymd_and_hms(2026, 9, 16, 0, 0, 0).unwrap(),
            excerpt: None,
            language: None,
            authors: Vec::new(),
            source: ArticleSource::from(source),
            topics,
        }
    }

    #[test]
    fn validates_request_bounds() {
        assert!(validate_request(&CollectRequest::default()).is_ok());
        assert!(matches!(
            validate_request(&CollectRequest {
                days: 0,
                ..CollectRequest::default()
            }),
            Err(CollectorError::InvalidDays(0))
        ));
        assert!(matches!(
            validate_request(&CollectRequest {
                limit: 501,
                ..CollectRequest::default()
            }),
            Err(CollectorError::InvalidLimit(501))
        ));
    }

    #[test]
    fn filters_by_date_topic_and_undated_policy() {
        let source = SourceDefinition {
            id: "test".into(),
            name: "Test".into(),
            feed_url: "https://example.com/feed".into(),
            homepage_url: "https://example.com".into(),
            kind: SourceKind::Official,
            default_topics: vec![Topic::Kotlin],
        };
        let now = Utc.with_ymd_and_hms(2026, 9, 16, 0, 0, 0).unwrap();
        let parsed = ParsedFeed {
            articles: vec![
                article(
                    "matching",
                    Some(now - TimeDelta::days(2)),
                    &source,
                    vec![Topic::Kmp],
                ),
                article(
                    "old",
                    Some(now - TimeDelta::days(10)),
                    &source,
                    vec![Topic::Kmp],
                ),
                article("undated", None, &source, vec![Topic::Kmp]),
                article(
                    "wrong-topic",
                    Some(now - TimeDelta::days(1)),
                    &source,
                    vec![Topic::Android],
                ),
            ],
            fetched_entries: 4,
            skipped_invalid: 0,
        };
        let request = CollectRequest {
            topics: vec![Topic::Kmp],
            ..CollectRequest::default()
        };

        let (report, filtered) =
            filter_source_articles(&source, parsed, &request, now - TimeDelta::days(7), now);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "matching");
        assert_eq!(report.skipped_undated, 1);
        assert_eq!(report.matched_entries, 1);
    }
}
