pub mod collector;
pub mod feed;
pub mod markdown;
pub mod mcp;
pub mod model;
pub mod slack;
pub mod sources;

pub use collector::{Collector, CollectorError};
pub use model::{ArticleCollection, CollectRequest, SourceKind, Topic};
pub use slack::{
    SlackFormCatalog, SlackFormatError, SlackFormatRequest, SlackFormattedPost, SlackPostRequest,
    SlackPostType, format_slack_post, slack_form_catalog,
};
