use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub id: String,
    pub feed_url: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub description: Option<String>,
    pub last_refreshed: Option<i64>,
    pub created_at: i64,
    /// Validators from the last successful fetch, replayed as a conditional GET.
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    /// Why the most recent refresh failed; cleared by the next success.
    pub last_refresh_error: Option<String>,
    /// Unix time before which the host asked us not to refetch (429 `Retry-After`).
    pub retry_after_until: Option<i64>,
}

impl Subscription {
    /// Fills in feed metadata a freshly parsed response lacks from what we already have,
    /// so a degraded or partial response (a trimmed feed from a CDN, one caught mid-edit)
    /// can't wipe good data. Storage upserts write the parsed values unconditionally, and
    /// `parse_feed` produces "no value" in two ways that would otherwise clobber:
    ///
    /// - `title` falls back to the feed URL when the response has no `<title>`. That
    ///   fallback is treated as "absent", not as a rename.
    /// - `artwork_url` and `description` are `None` when the response omits them.
    ///
    /// A value the response does provide always wins, so a publisher changing its title,
    /// artwork or description is still picked up. The cost is that a publisher *removing*
    /// its artwork or description is not noticed; a stale image or blurb is harmless and
    /// a lost one is not.
    pub fn inherit_missing_metadata(&mut self, existing: &Subscription) {
        if self.title == self.feed_url && !existing.title.is_empty() {
            self.title = existing.title.clone();
        }
        if self.artwork_url.is_none() {
            self.artwork_url = existing.artwork_url.clone();
        }
        if self.description.is_none() {
            self.description = existing.description.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subscription(title: &str, artwork: Option<&str>, description: Option<&str>) -> Subscription {
        Subscription {
            id: "id".to_string(),
            feed_url: "https://example.com/feed.rss".to_string(),
            title: title.to_string(),
            artwork_url: artwork.map(str::to_string),
            description: description.map(str::to_string),
            last_refreshed: None,
            created_at: 0,
            etag: None,
            last_modified: None,
            last_refresh_error: None,
            retry_after_until: None,
        }
    }

    #[test]
    fn a_response_without_metadata_keeps_the_known_values() {
        let existing = subscription("Real Title", Some("https://x/art.png"), Some("About"));
        // What `parse_feed` produces for a feed with no title, image or description.
        let mut fresh = subscription("https://example.com/feed.rss", None, None);

        fresh.inherit_missing_metadata(&existing);

        assert_eq!(fresh.title, "Real Title");
        assert_eq!(fresh.artwork_url.as_deref(), Some("https://x/art.png"));
        assert_eq!(fresh.description.as_deref(), Some("About"));
    }

    #[test]
    fn values_the_response_provides_always_win() {
        let existing = subscription("Old Title", Some("https://x/old.png"), Some("Old blurb"));
        let mut fresh = subscription("New Title", Some("https://x/new.png"), Some("New blurb"));

        fresh.inherit_missing_metadata(&existing);

        assert_eq!(fresh.title, "New Title");
        assert_eq!(fresh.artwork_url.as_deref(), Some("https://x/new.png"));
        assert_eq!(fresh.description.as_deref(), Some("New blurb"));
    }

    #[test]
    fn each_field_is_decided_independently() {
        let existing = subscription("Old Title", Some("https://x/old.png"), Some("Old blurb"));
        let mut fresh = subscription("New Title", None, Some("New blurb"));

        fresh.inherit_missing_metadata(&existing);

        assert_eq!(fresh.title, "New Title");
        assert_eq!(fresh.artwork_url.as_deref(), Some("https://x/old.png"));
        assert_eq!(fresh.description.as_deref(), Some("New blurb"));
    }

    #[test]
    fn nothing_is_invented_when_there_was_nothing_to_keep() {
        let existing = subscription("https://example.com/feed.rss", None, None);
        let mut fresh = subscription("https://example.com/feed.rss", None, None);

        fresh.inherit_missing_metadata(&existing);

        assert_eq!(fresh.title, "https://example.com/feed.rss");
        assert!(fresh.artwork_url.is_none());
        assert!(fresh.description.is_none());
    }

    #[test]
    fn an_empty_existing_title_is_not_preferred_over_the_url_fallback() {
        let existing = subscription("", None, None);
        let mut fresh = subscription("https://example.com/feed.rss", None, None);

        fresh.inherit_missing_metadata(&existing);

        assert_eq!(fresh.title, "https://example.com/feed.rss");
    }
}
