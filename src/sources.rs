use crate::model::{SourceDefinition, SourceKind, Topic};

pub fn built_in_sources() -> Vec<SourceDefinition> {
    vec![
        SourceDefinition {
            id: "android-developers-blog".into(),
            name: "Android Developers Blog".into(),
            feed_url: "https://developer.android.com/static/blog/atom.xml".into(),
            homepage_url: "https://developer.android.com/blog/".into(),
            kind: SourceKind::Official,
            default_topics: vec![Topic::Android],
        },
        SourceDefinition {
            id: "androidx-release-notes".into(),
            name: "AndroidX Release Notes".into(),
            feed_url: "https://developer.android.com/feeds/androidx-release-notes.xml".into(),
            homepage_url: "https://developer.android.com/jetpack/androidx/versions/all-channel"
                .into(),
            kind: SourceKind::Official,
            default_topics: vec![Topic::Android, Topic::Androidx, Topic::Jetpack],
        },
        SourceDefinition {
            id: "kotlin-blog".into(),
            name: "Kotlin Blog".into(),
            feed_url: "https://blog.jetbrains.com/kotlin/feed/".into(),
            homepage_url: "https://blog.jetbrains.com/kotlin/".into(),
            kind: SourceKind::Official,
            default_topics: vec![Topic::Kotlin],
        },
        SourceDefinition {
            id: "proandroiddev".into(),
            name: "ProAndroidDev".into(),
            feed_url: "https://proandroiddev.com/feed".into(),
            homepage_url: "https://proandroiddev.com/".into(),
            kind: SourceKind::Community,
            default_topics: vec![Topic::Android, Topic::Community],
        },
        SourceDefinition {
            id: "android-dagashi".into(),
            name: "AndroidDagashi".into(),
            feed_url: "https://feeds.feedburner.com/AndroidDagashi".into(),
            homepage_url: "https://androiddagashi.github.io/".into(),
            kind: SourceKind::Community,
            default_topics: vec![Topic::Android, Topic::Community],
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn source_ids_are_unique_and_urls_are_https() {
        let sources = built_in_sources();
        let ids = sources
            .iter()
            .map(|source| source.id.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(ids.len(), sources.len());
        assert!(
            sources
                .iter()
                .all(|source| source.feed_url.starts_with("https://"))
        );
    }
}
