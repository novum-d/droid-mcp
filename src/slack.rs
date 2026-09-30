use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::model::SCHEMA_VERSION;

const MAX_TITLE_CHARS: usize = 200;
const MAX_BODY_CHARS: usize = 2_800;
const MAX_URL_CHARS: usize = 3_000;
const MAX_FALLBACK_CHARS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SlackPostType {
    News,
    AndroidOs,
    Compose,
    Library,
    Question,
}

impl SlackPostType {
    fn emoji(self) -> &'static str {
        match self {
            Self::News => "📰",
            Self::AndroidOs => "🤖",
            Self::Compose => "🧩",
            Self::Library => "📦",
            Self::Question => "❓",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::News => "News投稿",
            Self::AndroidOs => "Android OS情報投稿",
            Self::Compose => "Compose投稿",
            Self::Library => "Library投稿",
            Self::Question => "質問・相談",
        }
    }

    fn message_template(self) -> &'static str {
        match self {
            Self::News => {
                "【📰 News投稿】\n{タイトル}\n────────────────────\n{概要}\n💬 ── コメント ──\n{コメント}\n🔗 ── 参考URL ──\n{URL}"
            }
            Self::AndroidOs => {
                "【🤖 Android OS情報投稿】\n{タイトル}\n────────────────────\n• OS: {対象OS}\n• Target: {対象targetSdk}\n👷 ── 変更内容 ──\n{変更内容}\n📲 ── アプリへの影響 ──\n{アプリへの影響}\n📝 ── 対応・調査結果 ──\n{対応・調査結果}\n🔗 ── 参考URL ──\n{URL}"
            }
            Self::Compose => {
                "【🧩 Compose投稿】\n{タイトル}\n────────────────────\n{概要}\n💬 ── サンプル・補足 ──\n{サンプル・補足}\n🔗 ── 参考URL ──\n{URL}"
            }
            Self::Library => {
                "【📦 Library投稿】\n{タイトル}\n────────────────────\n{概要}\n• ライブラリ名: {ライブラリ名}\n• バージョン: {バージョン}\n🔗 ── 参考URL ──\n{URL}"
            }
            Self::Question => {
                "【❓ 質問・相談】\n{タイトル}\n────────────────────\n{質問・困っていること}\n🏃 ── 試したこと ──\n{試したこと}\n💻 ── 動作環境 ──\n{動作環境}\n🔗 ── 参考URL ──\n{参考URL・コードなど}"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SlackInputKind {
    SingleLine,
    MultiLine,
    Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SlackFormField {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub input_kind: SlackInputKind,
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SlackPostTypeSpec {
    pub post_type: SlackPostType,
    pub name: String,
    pub emoji: String,
    pub message_template: String,
    pub fields: Vec<SlackFormField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SlackFormCatalog {
    pub schema_version: String,
    pub guidance: Vec<String>,
    pub post_types: Vec<SlackPostTypeSpec>,
}

/// Content for one of the five supported Slack post types.
///
/// The `postType` discriminator selects a variant and its required fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "postType",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum SlackPostRequest {
    News {
        title: String,
        url: String,
        summary: Option<String>,
        comment: Option<String>,
    },
    AndroidOs {
        title: String,
        target_os: Option<String>,
        target_sdk: Option<String>,
        change_details: String,
        app_impact: Option<String>,
        response_findings: Option<String>,
        reference_url: String,
    },
    Compose {
        title: String,
        summary: String,
        sample_notes: Option<String>,
        reference_url: Option<String>,
    },
    Library {
        library_name: String,
        title: String,
        version: Option<String>,
        summary: String,
        reference_url: Option<String>,
    },
    Question {
        title: String,
        question: String,
        tried: Option<String>,
        environment: Option<String>,
        references_or_code: Option<String>,
    },
}

/// MCP-compatible wrapper whose root JSON Schema is an object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SlackFormatRequest {
    pub post: SlackPostRequest,
}

impl SlackPostRequest {
    pub fn post_type(&self) -> SlackPostType {
        match self {
            Self::News { .. } => SlackPostType::News,
            Self::AndroidOs { .. } => SlackPostType::AndroidOs,
            Self::Compose { .. } => SlackPostType::Compose,
            Self::Library { .. } => SlackPostType::Library,
            Self::Question { .. } => SlackPostType::Question,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SlackTextType {
    Mrkdwn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SlackTextObject {
    #[serde(rename = "type")]
    pub kind: SlackTextType,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SlackBlock {
    Section { text: SlackTextObject },
    Divider,
}

/// The value of `message` can be passed to a Slack message API or incoming webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SlackMessagePayload {
    pub text: String,
    pub blocks: Vec<SlackBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SlackFormattedPost {
    pub schema_version: String,
    pub post_type: SlackPostType,
    pub message: SlackMessagePayload,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SlackFormatError {
    #[error("'{field}' is required")]
    Required { field: &'static str },
    #[error("'{field}' must not exceed {max} characters")]
    TooLong { field: &'static str, max: usize },
    #[error("'{field}' must be an absolute HTTP or HTTPS URL")]
    InvalidUrl { field: &'static str },
}

pub fn slack_form_catalog() -> SlackFormCatalog {
    SlackFormCatalog {
        schema_version: SCHEMA_VERSION.to_owned(),
        guidance: vec![
            "具体例がある項目は「例：…」で示します。".into(),
            "自由記述は「〜を簡単に記載してください」で示します。".into(),
            "URL項目は推奨する情報源の種類を示します。".into(),
            "News投稿の概要は記事の内容、コメントは投稿者自身の視点です。".into(),
        ],
        post_types: vec![
            spec(
                SlackPostType::News,
                vec![
                    field(
                        "title",
                        "タイトル",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Android 17 Beta 2 がリリース",
                    ),
                    field(
                        "url",
                        "URL",
                        true,
                        SlackInputKind::Url,
                        "共有する記事・公式ドキュメントなどのURL",
                    ),
                    field(
                        "summary",
                        "概要",
                        false,
                        SlackInputKind::MultiLine,
                        "記事の内容を1〜3行程度で簡単にまとめてください",
                    ),
                    field(
                        "comment",
                        "コメント",
                        false,
                        SlackInputKind::MultiLine,
                        "気になった点や、共有したいと思った理由など",
                    ),
                ],
            ),
            spec(
                SlackPostType::AndroidOs,
                vec![
                    field(
                        "title",
                        "タイトル",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Android 17で大画面のOrientation制限が変更",
                    ),
                    field(
                        "targetOs",
                        "対象OS",
                        false,
                        SlackInputKind::SingleLine,
                        "例：Android 17 / API 37",
                    ),
                    field(
                        "targetSdk",
                        "対象targetSdk",
                        false,
                        SlackInputKind::SingleLine,
                        "例：全アプリ / targetSdk 37以上",
                    ),
                    field(
                        "changeDetails",
                        "変更内容",
                        true,
                        SlackInputKind::MultiLine,
                        "従来の挙動から何が変わるのかを記載してください",
                    ),
                    field(
                        "appImpact",
                        "アプリへの影響",
                        false,
                        SlackInputKind::MultiLine,
                        "既存アプリで影響を受ける画面・機能・条件など",
                    ),
                    field(
                        "responseFindings",
                        "対応・調査結果",
                        false,
                        SlackInputKind::MultiLine,
                        "必要な対応、検証結果、現時点での判断など",
                    ),
                    field(
                        "referenceUrl",
                        "参考URL",
                        true,
                        SlackInputKind::Url,
                        "Android Developers / AOSP / Issue Trackerなどの一次情報を推奨",
                    ),
                ],
            ),
            spec(
                SlackPostType::Compose,
                vec![
                    field(
                        "title",
                        "タイトル",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Modifier.Nodeを使った独自Modifierの実装",
                    ),
                    field(
                        "summary",
                        "概要",
                        true,
                        SlackInputKind::MultiLine,
                        "何ができるのか、何が変わったのかを簡単に記載してください",
                    ),
                    field(
                        "sampleNotes",
                        "サンプル・補足",
                        false,
                        SlackInputKind::MultiLine,
                        "コード例、利用時の注意点、実際に試した感想など",
                    ),
                    field(
                        "referenceUrl",
                        "参考URL",
                        false,
                        SlackInputKind::Url,
                        "公式ドキュメント、Release Notes、記事、リポジトリなど",
                    ),
                ],
            ),
            spec(
                SlackPostType::Library,
                vec![
                    field(
                        "libraryName",
                        "ライブラリ名",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Coil / Retrofit / kotlinx.coroutines",
                    ),
                    field(
                        "title",
                        "タイトル",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Coil 3.4.0がリリース",
                    ),
                    field(
                        "version",
                        "バージョン",
                        false,
                        SlackInputKind::SingleLine,
                        "例：3.4.0",
                    ),
                    field(
                        "summary",
                        "概要",
                        true,
                        SlackInputKind::MultiLine,
                        "主な変更点、導入メリット、Breaking Changeなど",
                    ),
                    field(
                        "referenceUrl",
                        "参考URL",
                        false,
                        SlackInputKind::Url,
                        "GitHub Releases、公式ドキュメントなど",
                    ),
                ],
            ),
            spec(
                SlackPostType::Question,
                vec![
                    field(
                        "title",
                        "タイトル",
                        true,
                        SlackInputKind::SingleLine,
                        "例：Reader ModeとForeground Dispatchの使い分けについて",
                    ),
                    field(
                        "question",
                        "質問・困っていること",
                        true,
                        SlackInputKind::MultiLine,
                        "知りたいことや、現在困っていることを記載してください",
                    ),
                    field(
                        "tried",
                        "試したこと",
                        false,
                        SlackInputKind::MultiLine,
                        "調べたこと、試した実装、確認済みの内容など",
                    ),
                    field(
                        "environment",
                        "環境",
                        false,
                        SlackInputKind::SingleLine,
                        "例：Android 16 / Pixel 8 / Kotlin 2.x / Compose",
                    ),
                    field(
                        "referencesOrCode",
                        "参考URL・コードなど",
                        false,
                        SlackInputKind::MultiLine,
                        "関連ドキュメント、Issue、コード、エラーログなど",
                    ),
                ],
            ),
        ],
    }
}

pub fn format_slack_post(
    request: SlackPostRequest,
) -> Result<SlackFormattedPost, SlackFormatError> {
    let post_type = request.post_type();
    let mut blocks = Vec::new();
    let mut fallback_lines = Vec::new();

    let title = match &request {
        SlackPostRequest::News { title, .. }
        | SlackPostRequest::AndroidOs { title, .. }
        | SlackPostRequest::Compose { title, .. }
        | SlackPostRequest::Library { title, .. }
        | SlackPostRequest::Question { title, .. } => required(title, "title", MAX_TITLE_CHARS)?,
    };

    let heading = format!("【{} {}】", post_type.emoji(), post_type.label());
    blocks.push(section(format!("*{heading}*\n*{}*", escape_mrkdwn(title))));
    blocks.push(SlackBlock::Divider);
    fallback_lines.push(heading);
    fallback_lines.push(escape_mrkdwn(title));
    fallback_lines.push("────────────────────".into());

    match request {
        SlackPostRequest::News {
            url,
            summary,
            comment,
            ..
        } => {
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "概要",
                summary.as_deref(),
                None,
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "コメント",
                comment.as_deref(),
                Some("💬 ── コメント ──"),
            )?;
            url_section(
                &mut blocks,
                &mut fallback_lines,
                "URL",
                &url,
                Some("🔗 ── 参考URL ──"),
            )?;
        }
        SlackPostRequest::AndroidOs {
            target_os,
            target_sdk,
            change_details,
            app_impact,
            response_findings,
            reference_url,
            ..
        } => {
            optional_bullet(&mut blocks, &mut fallback_lines, "OS", target_os.as_deref())?;
            optional_bullet(
                &mut blocks,
                &mut fallback_lines,
                "Target",
                target_sdk.as_deref(),
            )?;
            required_text(
                &mut blocks,
                &mut fallback_lines,
                "changeDetails",
                &change_details,
                Some("👷 ── 変更内容 ──"),
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "アプリへの影響",
                app_impact.as_deref(),
                Some("📲 ── アプリへの影響 ──"),
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "対応・調査結果",
                response_findings.as_deref(),
                Some("📝 ── 対応・調査結果 ──"),
            )?;
            url_section(
                &mut blocks,
                &mut fallback_lines,
                "参考URL",
                &reference_url,
                Some("🔗 ── 参考URL ──"),
            )?;
        }
        SlackPostRequest::Compose {
            summary,
            sample_notes,
            reference_url,
            ..
        } => {
            required_text(&mut blocks, &mut fallback_lines, "summary", &summary, None)?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "サンプル・補足",
                sample_notes.as_deref(),
                Some("💬 ── サンプル・補足 ──"),
            )?;
            if let Some(url) = non_empty(reference_url.as_deref()) {
                url_section(
                    &mut blocks,
                    &mut fallback_lines,
                    "参考URL",
                    url,
                    Some("🔗 ── 参考URL ──"),
                )?;
            }
        }
        SlackPostRequest::Library {
            library_name,
            version,
            summary,
            reference_url,
            ..
        } => {
            required_text(&mut blocks, &mut fallback_lines, "summary", &summary, None)?;
            let library_name = required(&library_name, "libraryName", MAX_BODY_CHARS)?;
            push_bullet(
                &mut blocks,
                &mut fallback_lines,
                "ライブラリ名",
                library_name,
            );
            optional_bullet(
                &mut blocks,
                &mut fallback_lines,
                "バージョン",
                version.as_deref(),
            )?;
            if let Some(url) = non_empty(reference_url.as_deref()) {
                url_section(
                    &mut blocks,
                    &mut fallback_lines,
                    "参考URL",
                    url,
                    Some("🔗 ── 参考URL ──"),
                )?;
            }
        }
        SlackPostRequest::Question {
            question,
            tried,
            environment,
            references_or_code,
            ..
        } => {
            required_text(
                &mut blocks,
                &mut fallback_lines,
                "question",
                &question,
                None,
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "試したこと",
                tried.as_deref(),
                Some("🏃 ── 試したこと ──"),
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "環境",
                environment.as_deref(),
                Some("💻 ── 動作環境 ──"),
            )?;
            optional_text(
                &mut blocks,
                &mut fallback_lines,
                "参考URL・コードなど",
                references_or_code.as_deref(),
                Some("🔗 ── 参考URL ──"),
            )?;
        }
    }

    Ok(SlackFormattedPost {
        schema_version: SCHEMA_VERSION.to_owned(),
        post_type,
        message: SlackMessagePayload {
            text: truncate_chars(&fallback_lines.join("\n"), MAX_FALLBACK_CHARS),
            blocks,
        },
    })
}

fn spec(post_type: SlackPostType, fields: Vec<SlackFormField>) -> SlackPostTypeSpec {
    SlackPostTypeSpec {
        post_type,
        name: post_type.label().into(),
        emoji: post_type.emoji().into(),
        message_template: post_type.message_template().into(),
        fields,
    }
}

fn field(
    key: &str,
    label: &str,
    required: bool,
    input_kind: SlackInputKind,
    hint: &str,
) -> SlackFormField {
    SlackFormField {
        key: key.into(),
        label: label.into(),
        required,
        input_kind,
        hint: hint.into(),
    }
}

fn required<'a>(
    value: &'a str,
    field: &'static str,
    max: usize,
) -> Result<&'a str, SlackFormatError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SlackFormatError::Required { field });
    }
    validate_length(value, field, max)?;
    Ok(value)
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn validate_length(value: &str, field: &'static str, max: usize) -> Result<(), SlackFormatError> {
    if value.chars().count() > max {
        return Err(SlackFormatError::TooLong { field, max });
    }
    Ok(())
}

fn parse_url(value: &str, field: &'static str) -> Result<String, SlackFormatError> {
    let value = required(value, field, MAX_URL_CHARS)?;
    let url = Url::parse(value).map_err(|_| SlackFormatError::InvalidUrl { field })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(SlackFormatError::InvalidUrl { field });
    }
    Ok(url.into())
}

fn required_text(
    blocks: &mut Vec<SlackBlock>,
    fallback: &mut Vec<String>,
    field: &'static str,
    value: &str,
    heading: Option<&str>,
) -> Result<(), SlackFormatError> {
    let value = required(value, field, MAX_BODY_CHARS)?;
    push_text(blocks, fallback, value, heading);
    Ok(())
}

fn optional_text(
    blocks: &mut Vec<SlackBlock>,
    fallback: &mut Vec<String>,
    field: &'static str,
    value: Option<&str>,
    heading: Option<&str>,
) -> Result<(), SlackFormatError> {
    let Some(value) = non_empty(value) else {
        return Ok(());
    };
    validate_length(value, field, MAX_BODY_CHARS)?;
    push_text(blocks, fallback, value, heading);
    Ok(())
}

fn push_text(
    blocks: &mut Vec<SlackBlock>,
    fallback: &mut Vec<String>,
    value: &str,
    heading: Option<&str>,
) {
    let value = escape_mrkdwn(value);
    if let Some(heading) = heading {
        blocks.push(section(format!("*{heading}*\n{value}")));
        fallback.push(heading.to_owned());
    } else {
        blocks.push(section(value.clone()));
    }
    fallback.push(value);
}

fn optional_bullet(
    blocks: &mut Vec<SlackBlock>,
    fallback: &mut Vec<String>,
    label: &'static str,
    value: Option<&str>,
) -> Result<(), SlackFormatError> {
    let Some(value) = non_empty(value) else {
        return Ok(());
    };
    validate_length(value, label, MAX_BODY_CHARS)?;
    push_bullet(blocks, fallback, label, value);
    Ok(())
}

fn push_bullet(blocks: &mut Vec<SlackBlock>, fallback: &mut Vec<String>, label: &str, value: &str) {
    let line = format!("• {label}: {}", escape_mrkdwn(value));
    blocks.push(section(line.clone()));
    fallback.push(line);
}

fn url_section(
    blocks: &mut Vec<SlackBlock>,
    fallback: &mut Vec<String>,
    label: &'static str,
    value: &str,
    heading: Option<&str>,
) -> Result<(), SlackFormatError> {
    let url = parse_url(value, label)?;
    if let Some(heading) = heading {
        blocks.push(section(format!("*{heading}*\n<{url}|{url}>")));
        fallback.push(heading.to_owned());
    } else {
        blocks.push(section(format!("<{url}|{url}>")));
    }
    fallback.push(url);
    Ok(())
}

fn section(text: String) -> SlackBlock {
    SlackBlock::Section {
        text: SlackTextObject {
            kind: SlackTextType::Mrkdwn,
            text,
        },
    }
}

fn escape_mrkdwn(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let mut truncated = value
        .chars()
        .take(max.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_preserves_required_fields_and_hints() {
        let catalog = slack_form_catalog();
        assert_eq!(catalog.post_types.len(), 5);

        let news = &catalog.post_types[0];
        assert_eq!(news.post_type, SlackPostType::News);
        assert!(
            news.fields
                .iter()
                .find(|field| field.key == "title")
                .unwrap()
                .required
        );
        assert!(
            news.fields
                .iter()
                .find(|field| field.key == "url")
                .unwrap()
                .required
        );
        assert!(
            !news
                .fields
                .iter()
                .find(|field| field.key == "summary")
                .unwrap()
                .required
        );

        let android = &catalog.post_types[1];
        let target_sdk = android
            .fields
            .iter()
            .find(|field| field.key == "targetSdk")
            .unwrap();
        assert!(target_sdk.hint.contains("全アプリ"));
        assert!(target_sdk.hint.contains("targetSdk 37以上"));
    }

    #[test]
    fn news_keeps_summary_and_comment_separate_and_escapes_control_syntax() {
        let formatted = format_slack_post(SlackPostRequest::News {
            title: "Android <17>".into(),
            url: "https://developer.android.com/about/versions/17".into(),
            summary: Some("記事 & 公式情報".into()),
            comment: Some("<!channel> 要確認".into()),
        })
        .unwrap();

        assert_eq!(formatted.post_type, SlackPostType::News);
        let serialized = serde_json::to_string(&formatted.message).unwrap();
        assert!(serialized.contains("*💬 ── コメント ──*"));
        assert!(serialized.contains("記事 &amp; 公式情報"));
        assert!(serialized.contains("Android &lt;17&gt;"));
        assert!(serialized.contains("&lt;!channel&gt;"));
        assert!(serialized.contains("&amp;"));
        assert!(formatted.message.text.contains("&lt;!channel&gt;"));
    }

    #[test]
    fn required_values_and_urls_are_validated() {
        let missing_title = format_slack_post(SlackPostRequest::Compose {
            title: " ".into(),
            summary: "概要".into(),
            sample_notes: None,
            reference_url: None,
        });
        assert_eq!(
            missing_title,
            Err(SlackFormatError::Required { field: "title" })
        );

        let unsafe_url = format_slack_post(SlackPostRequest::News {
            title: "News".into(),
            url: "file:///etc/passwd".into(),
            summary: None,
            comment: None,
        });
        assert_eq!(
            unsafe_url,
            Err(SlackFormatError::InvalidUrl { field: "URL" })
        );
    }

    #[test]
    fn formats_each_post_type() {
        let requests = [
            SlackPostRequest::AndroidOs {
                title: "OS変更".into(),
                target_os: Some("Android 17 / API 37".into()),
                target_sdk: Some("全アプリ".into()),
                change_details: "挙動が変わります".into(),
                app_impact: None,
                response_findings: None,
                reference_url: "https://developer.android.com/".into(),
            },
            SlackPostRequest::Compose {
                title: "Compose".into(),
                summary: "概要".into(),
                sample_notes: None,
                reference_url: None,
            },
            SlackPostRequest::Library {
                library_name: "Coil".into(),
                title: "Coil update".into(),
                version: Some("3.4.0".into()),
                summary: "変更内容".into(),
                reference_url: Some("https://github.com/coil-kt/coil".into()),
            },
            SlackPostRequest::Question {
                title: "相談".into(),
                question: "使い分けを知りたい".into(),
                tried: None,
                environment: Some("Android 16".into()),
                references_or_code: None,
            },
        ];

        for request in requests {
            let formatted = format_slack_post(request).unwrap();
            assert!(!formatted.message.text.is_empty());
            assert!(formatted.message.blocks.len() >= 3);
        }
    }

    #[test]
    fn message_text_matches_workflow_templates() {
        let cases = [
            (
                SlackPostRequest::News {
                    title: "新着記事".into(),
                    url: "https://example.com/news".into(),
                    summary: Some("記事の概要".into()),
                    comment: Some("確認したい点".into()),
                },
                "【📰 News投稿】\n新着記事\n────────────────────\n記事の概要\n💬 ── コメント ──\n確認したい点\n🔗 ── 参考URL ──\nhttps://example.com/news",
            ),
            (
                SlackPostRequest::AndroidOs {
                    title: "OS変更".into(),
                    target_os: Some("Android 17".into()),
                    target_sdk: Some("全アプリ".into()),
                    change_details: "変更内容".into(),
                    app_impact: Some("影響".into()),
                    response_findings: Some("調査中".into()),
                    reference_url: "https://example.com/os".into(),
                },
                "【🤖 Android OS情報投稿】\nOS変更\n────────────────────\n• OS: Android 17\n• Target: 全アプリ\n👷 ── 変更内容 ──\n変更内容\n📲 ── アプリへの影響 ──\n影響\n📝 ── 対応・調査結果 ──\n調査中\n🔗 ── 参考URL ──\nhttps://example.com/os",
            ),
            (
                SlackPostRequest::Compose {
                    title: "Compose更新".into(),
                    summary: "概要".into(),
                    sample_notes: Some("サンプル".into()),
                    reference_url: Some("https://example.com/compose".into()),
                },
                "【🧩 Compose投稿】\nCompose更新\n────────────────────\n概要\n💬 ── サンプル・補足 ──\nサンプル\n🔗 ── 参考URL ──\nhttps://example.com/compose",
            ),
            (
                SlackPostRequest::Library {
                    library_name: "Coil".into(),
                    title: "Coil更新".into(),
                    version: Some("3.4.0".into()),
                    summary: "概要".into(),
                    reference_url: Some("https://example.com/library".into()),
                },
                "【📦 Library投稿】\nCoil更新\n────────────────────\n概要\n• ライブラリ名: Coil\n• バージョン: 3.4.0\n🔗 ── 参考URL ──\nhttps://example.com/library",
            ),
            (
                SlackPostRequest::Question {
                    title: "相談".into(),
                    question: "質問内容".into(),
                    tried: Some("試したこと".into()),
                    environment: Some("Android 16".into()),
                    references_or_code: Some("関連コード".into()),
                },
                "【❓ 質問・相談】\n相談\n────────────────────\n質問内容\n🏃 ── 試したこと ──\n試したこと\n💻 ── 動作環境 ──\nAndroid 16\n🔗 ── 参考URL ──\n関連コード",
            ),
        ];

        for (request, expected) in cases {
            let formatted = format_slack_post(request).unwrap();
            assert_eq!(formatted.message.text, expected);
        }
    }
}
