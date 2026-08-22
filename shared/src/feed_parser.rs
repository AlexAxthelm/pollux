use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use feed_rs::parser;

use crate::domain::{DownloadStatus, Episode, PlaybackStatus, Subscription};

pub fn parse_feed(url: &str, body: Vec<u8>) -> Result<(Subscription, Vec<Episode>), String> {
    let feed = parser::parse(body.as_slice()).map_err(|e| e.to_string())?;

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
            let duration_secs = media
                .duration
                .map(|d| d.as_secs().min(u32::MAX as u64) as u32);
            let artwork_url = media.thumbnails.into_iter().next().map(|t| t.image.uri);
            let content = media.content.into_iter().next()?;
            let enclosure_url = content.url?.to_string();
            let file_size_bytes = content.size;

            let feed_guid = entry.id;
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
                title: entry
                    .title
                    .map(|t| t.content)
                    .unwrap_or_else(|| "Untitled".to_string()),
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

}
