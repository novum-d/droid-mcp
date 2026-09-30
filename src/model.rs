use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1.0";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Ord, PartialOrd,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Official,
    Community,
}

impl fmt::Display for SourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Official => formatter.write_str("official"),
            Self::Community => formatter.write_str("community"),
        }
    }
}

impl FromStr for SourceKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "official" => Ok(Self::Official),
            "community" => Ok(Self::Community),
            _ => Err(format!(
                "unknown source kind '{value}'; expected official or community"
            )),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Ord, PartialOrd,
)]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    Android,
    AndroidPlatform,
    Androidx,
    Jetpack,
    Compose,
    Kotlin,
    Kmp,
    Cmp,
    KotlinConf,
    Gradle,
    Agp,
    Tools,
    Community,
}

impl fmt::Display for Topic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Android => "android",
            Self::AndroidPlatform => "android_platform",
            Self::Androidx => "androidx",
            Self::Jetpack => "jetpack",
            Self::Compose => "compose",
            Self::Kotlin => "kotlin",
            Self::Kmp => "kmp",
            Self::Cmp => "cmp",
            Self::KotlinConf => "kotlin_conf",
            Self::Gradle => "gradle",
            Self::Agp => "agp",
            Self::Tools => "tools",
            Self::Community => "community",
        })
    }
}

impl FromStr for Topic {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "android" => Ok(Self::Android),
            "android_platform" | "platform" => Ok(Self::AndroidPlatform),
            "androidx" => Ok(Self::Androidx),
            "jetpack" => Ok(Self::Jetpack),
            "compose" | "jetpack_compose" => Ok(Self::Compose),
            "kotlin" => Ok(Self::Kotlin),
            "kmp" | "kotlin_multiplatform" => Ok(Self::Kmp),
            "cmp" | "compose_multiplatform" => Ok(Self::Cmp),
            "kotlin_conf" | "kotlinconf" => Ok(Self::KotlinConf),
            "gradle" => Ok(Self::Gradle),
            "agp" | "android_gradle_plugin" => Ok(Self::Agp),
            "tools" | "tooling" => Ok(Self::Tools),
            "community" => Ok(Self::Community),
            _ => Err(format!("unknown topic '{value}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceDefinition {
    pub id: String,
    pub name: String,
    pub feed_url: String,
    pub homepage_url: String,
    pub kind: SourceKind,
    pub default_topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArticleSource {
    pub id: String,
    pub name: String,
    pub kind: SourceKind,
}

impl From<&SourceDefinition> for ArticleSource {
    fn from(source: &SourceDefinition) -> Self {
        Self {
            id: source.id.clone(),
            name: source.name.clone(),
            kind: source.kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    /// Deterministic identifier derived from the normalized URL and title.
    pub id: String,
    pub title: String,
    pub url: String,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    /// Plain-text excerpt supplied by the source feed. This is not an AI summary.
    pub excerpt: Option<String>,
    pub language: Option<String>,
    pub authors: Vec<String>,
    pub source: ArticleSource,
    pub topics: Vec<Topic>,
}

impl Article {
    pub fn effective_date(&self) -> Option<DateTime<Utc>> {
        self.published_at.or(self.updated_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectRequest {
    /// Number of elapsed 24-hour periods to include. Allowed range: 1..=365.
    pub days: u32,
    /// Optional source IDs. An empty list selects all registered sources.
    pub source_ids: Vec<String>,
    /// Optional topic filter. An article matches when it has any requested topic.
    pub topics: Vec<Topic>,
    /// Optional source-kind filter. An empty list includes official and community sources.
    pub source_kinds: Vec<SourceKind>,
    /// Maximum number of articles in the response. Allowed range: 1..=500.
    pub limit: usize,
    /// Include feed entries without a usable published or updated timestamp.
    pub include_undated: bool,
}

impl Default for CollectRequest {
    fn default() -> Self {
        Self {
            days: 7,
            source_ids: Vec::new(),
            topics: Vec::new(),
            source_kinds: Vec::new(),
            limit: 100,
            include_undated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueryEcho {
    pub days: u32,
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
    pub source_ids: Vec<String>,
    pub topics: Vec<Topic>,
    pub source_kinds: Vec<SourceKind>,
    pub limit: usize,
    pub include_undated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceErrorCode {
    FetchFailed,
    ResponseTooLarge,
    UnsupportedContentType,
    ParseFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceError {
    pub source_id: String,
    pub code: SourceErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceReport {
    pub source_id: String,
    pub fetched_entries: usize,
    pub matched_entries: usize,
    pub skipped_undated: usize,
    pub skipped_invalid: usize,
    pub oldest_available_at: Option<DateTime<Utc>>,
    pub newest_available_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStats {
    pub fetched_entries: usize,
    pub matched_before_limit: usize,
    pub returned_articles: usize,
    pub duplicates_removed: usize,
    pub skipped_undated: usize,
    pub skipped_invalid: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArticleCollection {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub query: QueryEcho,
    pub articles: Vec<Article>,
    pub source_reports: Vec<SourceReport>,
    pub source_errors: Vec<SourceError>,
    pub stats: CollectionStats,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceList {
    pub schema_version: String,
    pub sources: Vec<SourceDefinition>,
}
