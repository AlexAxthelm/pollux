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

    let episodes = feed
        .entries
        .into_iter()
        .filter_map(|entry| {
            let media = entry.media.into_iter().next()?;

            // Extract fields before consuming media
            let duration_secs = media
                .duration
                .map(|d| d.as_secs().min(u32::MAX as u64) as u32);
            let artwork_url = media.thumbnails.into_iter().next().map(|t| t.image.uri);
            let content = media.content.into_iter().next()?;
            let enclosure_url = content.url?.to_string();
            let file_size_bytes = content.size;

            let description = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body));

            let pub_date = entry.published.map(|dt| dt.timestamp());

            Some(Episode {
                id: uuid::Uuid::new_v4().to_string(),
                feed_guid: entry.id,
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
