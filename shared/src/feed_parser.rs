use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use feed_rs::parser::Builder;

use crate::domain::{DownloadStatus, Episode, PlaybackStatus, Subscription};

/// Sentinel our id generator returns for a feed entry that carries no
/// `<guid>`/`<id>`. feed-rs invokes the generator *only* for entries missing an
/// id (see `assign_missing_ids`), so seeing this value downstream is a reliable,
/// documented-API signal that identity must be derived locally — rather than
/// guessing from the shape of feed-rs's own synthesised id.
const MISSING_GUID: &str = "\u{0}pollux:missing-guid\u{0}";

pub fn parse_feed(url: &str, body: Vec<u8>) -> Result<(Subscription, Vec<Episode>), String> {
    let feed = Builder::new()
        .id_generator(|_links, _title, _uri| MISSING_GUID.to_string())
        .build()
        .parse(body.as_slice())
        .map_err(|e| e.to_string())?;

    let now = now_unix();
    let subscription_id = uuid::Uuid::new_v4().to_string();

    let subscription = Subscription {
        id: subscription_id.clone(),
        feed_url: url.to_string(),
        title: feed
            .title
            .map(|t| t.content)
            .unwrap_or_else(|| url.to_string()),
        description: feed.description.map(|d| d.content),
        artwork_url: feed.logo.map(|img| img.uri),
        last_refreshed: Some(now),
        created_at: now,
    };

    // Episode identity is (subscription, feed_guid) in storage, where a repeat
    // guid silently overwrites the earlier row. Feeds are conventionally newest
    // first, so an unguarded overwrite would let an older duplicate replace a
    // newer episode. Collapse duplicates here instead, keeping the first.
    let mut seen_guids: HashSet<String> = HashSet::new();

    let episodes = feed
        .entries
        .into_iter()
        .filter_map(|entry| {
            let media = entry.media.into_iter().next()?;

            // Pull the fields we need out of media before moving past it.
            // The fields we're moving here are tracked individually by rust (borrow checking
            // shouldn't be a problem) and let us bail out of processing an entry early if there's
            // nothing there to look at (no url, no content, etc.).
            let duration_secs = media
                .duration
                .map(|d| d.as_secs().min(u32::MAX as u64) as u32);
            let artwork_url = media.thumbnails.into_iter().next().map(|t| t.image.uri);
            let content = media.content.into_iter().next()?;
            let enclosure_url = content.url?.to_string();
            let file_size_bytes = content.size;

            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string());

            // When the feed supplied a real guid, feed-rs hands it through as
            // entry.id and we use it verbatim. When it did not, our generator
            // stamped MISSING_GUID, and we derive a stable local id instead.
            let feed_guid = if entry.id == MISSING_GUID {
                stable_episode_id(&enclosure_url, &title)
            } else {
                entry.id
            };
            if !seen_guids.insert(feed_guid.clone()) {
                return None;
            }

            let description = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body));

            let pub_date = entry.published.map(|dt| dt.timestamp());

            Some(Episode {
                id: uuid::Uuid::new_v4().to_string(),
                feed_guid,
                subscription_id: subscription_id.clone(),
                title,
                description,
                pub_date,
                duration_secs,
                enclosure_url,
                artwork_url,
                playback_status: PlaybackStatus::Unplayed,
                playback_position_secs: None,
                download_status: DownloadStatus::NotDownloaded,
                download_progress: None,
                is_flagged: false,
                file_size_bytes,
                local_path: None,
            })
        })
        .collect();

    Ok((subscription, episodes))
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Derives a stable episode id for a feed entry that carries no `<guid>`/`<id>`.
///
/// Keyed on the enclosure URL and title, which together identify an episode
/// across refreshes. NOTE: it is deliberately *not* keyed on the parse-time
/// `subscription_id` — that is a fresh UUID on every parse, so folding it in
/// would make the id change every refresh, which is the exact bug this avoids.
/// Storage scopes uniqueness by `(subscription_id, feed_guid)`, so the derived
/// value only has to be unique within one feed.
///
/// Failure modes (acceptable, and milder than the alternatives): editing an
/// episode's title or a change to its enclosure URL yields a new id and thus a
/// duplicate row; two entries that share both a title and an enclosure URL
/// collapse to one. Both are rarer than the every-refresh churn of relying on
/// feed-rs's synthesised id.
fn stable_episode_id(enclosure_url: &str, title: &str) -> String {
    // Length-delimited so ("ab", "c") and ("a", "bc") cannot collide. The
    // version prefix lets us intentionally rev the scheme later without
    // silently re-keying existing episodes.
    let mut buf = Vec::new();
    buf.extend_from_slice(b"pollux-episode-v1\0");
    buf.extend_from_slice(enclosure_url.as_bytes());
    buf.push(0);
    buf.extend_from_slice(title.as_bytes());
    format!("{:016x}", fnv1a_64(&buf))
}

/// FNV-1a (64-bit). A small, fully specified hash chosen on purpose: unlike
/// std's `DefaultHasher`, whose algorithm std explicitly does not guarantee
/// across releases, FNV-1a's output is fixed forever — so episode ids written
/// to storage stay valid across Rust and app-version upgrades.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://example.com/feed.rss";

    fn rss_with_items(items: &str) -> Vec<u8> {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
  <title>Test Podcast</title>
  <link>https://example.com</link>
  {items}
</channel>
</rss>"#
        )
        .into_bytes()
    }

    fn parse_episodes(body: Vec<u8>) -> Vec<Episode> {
        parse_feed(URL, body).expect("feed should parse").1
    }

    #[test]
    fn duplicate_guids_keep_the_first_entry() {
        // Feeds list newest first, so the first occurrence is the one to keep.
        let episodes = parse_episodes(rss_with_items(
            r#"
  <item>
    <title>Newer</title>
    <guid>shared-guid</guid>
    <enclosure url="https://example.com/new.mp3" type="audio/mpeg" length="100"/>
  </item>
  <item>
    <title>Older</title>
    <guid>shared-guid</guid>
    <enclosure url="https://example.com/old.mp3" type="audio/mpeg" length="200"/>
  </item>"#,
        ));

        assert_eq!(episodes.len(), 1, "duplicate guid should collapse to one");
        assert_eq!(
            episodes[0].title, "Newer",
            "a later duplicate must not displace the earlier entry"
        );
        assert_eq!(episodes[0].enclosure_url, "https://example.com/new.mp3");
    }

    #[test]
    fn distinct_guids_are_all_kept() {
        let episodes = parse_episodes(rss_with_items(
            r#"
  <item>
    <title>One</title>
    <guid>guid-1</guid>
    <enclosure url="https://example.com/1.mp3" type="audio/mpeg" length="100"/>
  </item>
  <item>
    <title>Two</title>
    <guid>guid-2</guid>
    <enclosure url="https://example.com/2.mp3" type="audio/mpeg" length="200"/>
  </item>"#,
        ));

        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].title, "One");
        assert_eq!(episodes[1].title, "Two");
    }

    #[test]
    fn pub_date_is_read_from_rss_pubdate() {
        let episodes = parse_episodes(rss_with_items(
            r#"
  <item>
    <title>Dated</title>
    <guid>guid-1</guid>
    <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    <enclosure url="https://example.com/1.mp3" type="audio/mpeg" length="100"/>
  </item>"#,
        ));

        assert_eq!(episodes[0].pub_date, Some(1_704_067_200));
    }

    // -- Episode identity when <guid> is absent -----------------------------

    /// An item WITH a guid must use it verbatim — never the derived fallback.
    #[test]
    fn guid_present_is_used_verbatim() {
        let episodes = parse_episodes(rss_with_items(
            r#"
  <item>
    <title>Ep</title>
    <guid>real-guid-123</guid>
    <enclosure url="https://example.com/a.mp3" type="audio/mpeg" length="1"/>
  </item>"#,
        ));

        assert_eq!(episodes[0].feed_guid, "real-guid-123");
    }

    /// A guid-LESS item must get the same id every parse, so refreshes update
    /// the existing row instead of inserting a duplicate.
    #[test]
    fn missing_guid_is_stable_across_parses() {
        let item = r#"
  <item>
    <title>No Guid Here</title>
    <enclosure url="https://example.com/a.mp3" type="audio/mpeg" length="1"/>
  </item>"#;

        let first = parse_episodes(rss_with_items(item));
        let second = parse_episodes(rss_with_items(item));

        assert_ne!(
            first[0].feed_guid, MISSING_GUID,
            "the sentinel must be replaced by a derived id, not stored"
        );
        assert_eq!(
            first[0].feed_guid, second[0].feed_guid,
            "a guid-less entry must hash to the same id on every parse"
        );
    }

    /// The derived id must survive enclosure-URL rewriting is NOT claimed here —
    /// distinct enclosures are distinct episodes and must not collapse.
    #[test]
    fn missing_guid_distinct_enclosures_stay_distinct() {
        let episodes = parse_episodes(rss_with_items(
            r#"
  <item>
    <title>Same Title</title>
    <enclosure url="https://example.com/1.mp3" type="audio/mpeg" length="1"/>
  </item>
  <item>
    <title>Same Title</title>
    <enclosure url="https://example.com/2.mp3" type="audio/mpeg" length="1"/>
  </item>"#,
        ));

        assert_eq!(
            episodes.len(),
            2,
            "different enclosures are different episodes"
        );
        assert_ne!(episodes[0].feed_guid, episodes[1].feed_guid);
    }

    #[test]
    fn stable_episode_id_is_deterministic_and_input_sensitive() {
        let base = stable_episode_id("https://example.com/a.mp3", "Title");
        assert_eq!(
            base,
            stable_episode_id("https://example.com/a.mp3", "Title")
        );
        assert_ne!(
            base,
            stable_episode_id("https://example.com/b.mp3", "Title")
        );
        assert_ne!(
            base,
            stable_episode_id("https://example.com/a.mp3", "Other")
        );
        // Length-delimited framing: the split between the two fields matters.
        assert_ne!(
            stable_episode_id("ab", "c"),
            stable_episode_id("a", "bc"),
            "field boundary must not be ambiguous"
        );
    }

    #[test]
    fn fnv1a_matches_known_vector() {
        // FNV-1a/64 of "" and "a" are fixed reference values; locking them
        // guarantees the algorithm never silently drifts.
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }
}
