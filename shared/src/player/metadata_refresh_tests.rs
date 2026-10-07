//! The active episode keeps its own copy of the episode, so a feed refresh that corrects
//! its audio URL (or title, artwork) has to reach that copy, or the next load would use
//! the old one. Position and status are the player's own and must survive. See the module
//! docs in `player.rs`.

use super::tests::{active, episode, model_with, player_ops, send, session};
use super::*;

/// What the refreshed feed now says about "e1": a renewed audio URL and a new title.
fn refreshed() -> Episode {
    let mut e = episode("e1");
    e.enclosure_url = "https://cdn.example.com/renewed/e1.mp3".into();
    e.title = "Episode e1 (corrected)".into();
    e.duration_secs = Some(900);
    e
}

fn reload_list(model: &mut Model, rows: Vec<Episode>) {
    model.selected_subscription = model.subscriptions.first().cloned();
    send(
        model,
        Event::EpisodesLoaded {
            subscription_id: "sub".into(),
            result: Box::new(StorageResult::Episodes(rows)),
        },
    );
}

fn fail_the_stream(model: &mut Model) {
    let session = session(model);
    send(
        model,
        Event::PlayerFailed {
            session,
            message: "403".into(),
        },
    );
}

fn load_url(effects: &[Effect]) -> Option<String> {
    player_ops(effects).into_iter().find_map(|op| match op {
        PlayerOperation::Load {
            media: MediaSource::Stream { url },
            ..
        } => Some(url),
        _ => None,
    })
}

#[test]
fn retrying_after_a_failure_loads_the_renewed_url() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    fail_the_stream(&mut model);

    // The refresh brings a working URL; the list reloads with it.
    reload_list(&mut model, vec![refreshed()]);
    let effects = send(&mut model, Event::PlayEpisode("e1".into()));

    assert_eq!(
        load_url(&effects).as_deref(),
        Some("https://cdn.example.com/renewed/e1.mp3")
    );
}

#[test]
fn retrying_from_the_mini_player_loads_the_renewed_url_too() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    fail_the_stream(&mut model);
    reload_list(&mut model, vec![refreshed()]);

    let effects = send(&mut model, Event::TogglePlay);

    assert_eq!(
        load_url(&effects).as_deref(),
        Some("https://cdn.example.com/renewed/e1.mp3")
    );
}

#[test]
fn a_reload_keeps_the_position_and_status_of_the_active_episode() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    super::tests::tick(&mut model, 120);

    reload_list(&mut model, vec![refreshed()]);

    let a = active(&model);
    assert_eq!(a.position_secs, 120);
    assert_eq!(a.episode.playback_status, PlaybackStatus::InProgress);
    assert_eq!(a.episode.title, "Episode e1 (corrected)");
    assert_eq!(a.episode.duration_secs, Some(900));
    assert!(a.is_playing, "a refresh must not interrupt playback");
}

#[test]
fn a_reload_does_not_reload_the_engine() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    model.selected_subscription = model.subscriptions.first().cloned();

    let effects = send(
        &mut model,
        Event::EpisodesLoaded {
            subscription_id: "sub".into(),
            result: Box::new(StorageResult::Episodes(vec![refreshed()])),
        },
    );

    assert!(
        player_ops(&effects).is_empty(),
        "the new URL is for the next load, not a reason to interrupt this one"
    );
}

#[test]
fn a_downloaded_file_is_not_swapped_for_the_streams_new_url() {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/e1.mp3".into());
    let mut model = model_with(vec![e.clone()]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let mut fresh = refreshed();
    fresh.download_status = DownloadStatus::Downloaded;
    fresh.local_path = Some("Downloads/e1.mp3".into());
    reload_list(&mut model, vec![fresh]);

    assert!(matches!(
        active(&model).media,
        MediaSource::Local { ref local_path } if local_path == "Downloads/e1.mp3"
    ));
}

#[test]
fn other_episodes_in_the_reloaded_list_do_not_touch_the_active_one() {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let mut other = episode("e2");
    other.title = "Something else".into();
    reload_list(&mut model, vec![other]);

    assert_eq!(active(&model).episode.title, "Episode e1");
}
