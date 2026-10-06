use crux_core::App;

use super::*;
use crate::capabilities::download::DownloadResult;
use crate::domain::Subscription;
use crate::Pollux;

pub(super) fn episode(id: &str) -> Episode {
    Episode {
        id: id.to_string(),
        feed_guid: format!("{id}-guid"),
        subscription_id: "sub".to_string(),
        title: format!("Episode {id}"),
        description: None,
        pub_date: Some(1),
        duration_secs: Some(600),
        enclosure_url: format!("https://example.com/{id}.mp3"),
        artwork_url: None,
        playback_status: PlaybackStatus::Unplayed,
        playback_position_secs: None,
        download_status: DownloadStatus::NotDownloaded,
        is_flagged: false,
        file_size_bytes: None,
        local_path: None,
    }
}

pub(super) fn model_with(episodes: Vec<Episode>) -> Model {
    Model {
        subscriptions: vec![Subscription {
            id: "sub".to_string(),
            feed_url: "https://example.com/f.rss".to_string(),
            title: "Feed".to_string(),
            artwork_url: Some("https://example.com/feed.jpg".to_string()),
            description: None,
            last_refreshed: None,
            created_at: 0,
            etag: None,
            last_modified: None,
            last_refresh_error: None,
            retry_after_until: None,
        }],
        episodes,
        ..Model::default()
    }
}

pub(super) fn send(model: &mut Model, event: Event) -> Vec<Effect> {
    Pollux.update(event, model).effects().collect()
}

/// What the shell answers `LoadPlayContext` with: the saved episode and where it was
/// started from (the subscription the test episodes belong to).
pub(super) fn saved_context(episode: Episode) -> Event {
    Event::PlayContextLoaded(Box::new(StorageResult::PlayContext {
        episode,
        source: EpisodeSource::Subscription { id: "sub".into() },
    }))
}

/// The current load's session, which the shell tags every engine event with.
pub(super) fn session(model: &Model) -> u32 {
    active(model).session
}

/// A position report from the engine for the current load.
pub(super) fn tick(model: &mut Model, secs: u32) -> Vec<Effect> {
    let session = session(model);
    send(
        model,
        Event::PlayerTick {
            session,
            position_secs: secs,
        },
    )
}

/// The engine reaching the end of the current load.
pub(super) fn send_ended(model: &mut Model) -> Vec<Effect> {
    let session = session(model);
    send(model, Event::PlayerEnded { session })
}

pub(super) fn player_ops(effects: &[Effect]) -> Vec<PlayerOperation> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Player(r) => Some(r.operation.clone()),
            _ => None,
        })
        .collect()
}

pub(super) fn storage_ops(effects: &[Effect]) -> Vec<StorageOperation> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Storage(r) => Some(r.operation.clone()),
            _ => None,
        })
        .collect()
}

pub(super) fn checkpoints(effects: &[Effect]) -> Vec<(PlaybackStatus, Option<u32>)> {
    storage_ops(effects)
        .into_iter()
        .filter_map(|op| match op {
            StorageOperation::UpdatePlaybackStatus {
                status,
                position_secs,
                ..
            } => Some((status, position_secs)),
            _ => None,
        })
        .collect()
}

pub(super) fn active(model: &Model) -> &ActivePlayback {
    model.active_playback.as_ref().expect("an active episode")
}

pub(super) fn view_player(model: &Model) -> PlayerView {
    Pollux.view(model).player.expect("a player view")
}

#[test]
fn playing_an_undownloaded_episode_streams_and_starts_a_download() {
    let mut model = model_with(vec![episode("e1")]);
    let effects = send(&mut model, Event::PlayEpisode("e1".into()));

    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load {
            media: MediaSource::Stream { .. },
            start_secs: 0,
            autoplay: true,
            ..
        }]
    ));
    assert_eq!(model.downloading.as_deref(), Some("e1"));
    assert!(active(&model).is_playing);
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );
    assert!(storage_ops(&effects)
        .iter()
        .any(|op| matches!(op, StorageOperation::SavePlayContext { .. })));
}

#[test]
fn playing_a_downloaded_episode_uses_the_file_and_resumes_with_a_rewind() {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/abc.mp3".into());
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(100);
    let mut model = model_with(vec![e]);

    let effects = send(&mut model, Event::PlayEpisode("e1".into()));

    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load {
            media: MediaSource::Local { .. },
            start_secs: 97,
            ..
        }]
    ));
    assert!(model.downloading.is_none(), "no download needed");
}

#[test]
fn a_played_episode_restarts_from_the_top() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::Played;
    e.playback_position_secs = Some(0);
    let mut model = model_with(vec![e]);
    let effects = send(&mut model, Event::PlayEpisode("e1".into()));
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load { start_secs: 0, .. }]
    ));
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );
}

#[test]
fn pause_then_play_rewinds_and_persists_position() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 50);

    let effects = send(&mut model, Event::Pause);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Pause]
    ));
    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::InProgress, Some(50))]
    );

    let effects = send(&mut model, Event::Play);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { secs: 47 }, PlayerOperation::Play]
    ));
    assert_eq!(active(&model).position_secs, 47);
}

#[test]
fn rewind_is_clamped_at_zero() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, Event::Pause);
    send(&mut model, Event::Play);
    assert_eq!(active(&model).position_secs, 0);
}

#[test]
fn ticks_persist_only_every_checkpoint_interval() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    assert!(checkpoints(&tick(&mut model, 3)).is_empty());
    assert!(checkpoints(&tick(&mut model, 9)).is_empty());
    assert_eq!(
        checkpoints(&tick(&mut model, 10)),
        vec![(PlaybackStatus::InProgress, Some(10))]
    );
    assert!(checkpoints(&tick(&mut model, 12)).is_empty());
}

#[test]
fn skips_use_the_default_increments_and_clamp_to_the_episode() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let effects = send(&mut model, Event::SkipForward);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { secs: 30 }]
    ));

    // Back from 30 → 15, then the next back clamps at 0 rather than wrapping.
    send(&mut model, Event::SkipBack);
    assert_eq!(active(&model).position_secs, 15);
    send(&mut model, Event::SkipBack);
    send(&mut model, Event::SkipBack);
    assert_eq!(active(&model).position_secs, 0);

    // Skipping forward from near the end never overshoots into another episode: it
    // finishes this one (see `played_tests`).
    send(&mut model, Event::SeekTo(595));
    send(&mut model, Event::SkipForward);
    assert!(model.active_playback.is_none());
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
}

#[test]
fn engine_duration_overrides_the_feed_duration() {
    let mut e = episode("e1");
    e.duration_secs = None;
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let session = session(&model);
    send(
        &mut model,
        Event::PlayerDuration {
            session,
            duration_secs: 300,
        },
    );
    assert_eq!(view_player(&model).duration_secs, Some(300));
}

#[test]
fn reaching_the_end_marks_played_and_clears_the_player() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let effects = send_ended(&mut model);

    assert!(model.active_playback.is_none());
    assert!(Pollux.view(&model).player.is_none());
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
    assert!(storage_ops(&effects)
        .iter()
        .any(|op| matches!(op, StorageOperation::ClearPlayContext)));
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Stop]
    ));
}

#[test]
fn starting_another_episode_checkpoints_the_first() {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 42);
    let effects = send(&mut model, Event::PlayEpisode("e2".into()));
    assert!(storage_ops(&effects).iter().any(|op| matches!(
        op,
        StorageOperation::UpdatePlaybackStatus { episode_id, position_secs: Some(42), .. }
            if episode_id == "e1"
    )));
    assert_eq!(active(&model).episode.id, "e2");
}

#[test]
fn a_finished_download_swaps_the_stream_for_the_local_file_at_the_playhead() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 77);

    let effects = send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Completed {
                local_path: "Downloads/x.mp3".into(),
                size_bytes: 10,
            }),
        },
    );

    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load {
            media: MediaSource::Local { .. },
            start_secs: 77,
            autoplay: true,
            ..
        }]
    ));
    assert!(!view_player(&model).is_streaming);
}

#[test]
fn a_failed_stream_pauses_with_a_notice_and_play_reloads() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let session = session(&model);
    send(
        &mut model,
        Event::PlayerFailed {
            session,
            message: "offline".into(),
        },
    );
    let view = view_player(&model);
    assert!(!view.is_playing);
    assert_eq!(view.error.as_deref(), Some("offline"));

    let effects = send(&mut model, Event::Play);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load { .. }]
    ));
    assert!(view_player(&model).error.is_none());
}

#[test]
fn a_missing_local_file_falls_back_to_streaming_and_redownloads() {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/gone.mp3".into());
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let session = session(&model);
    let effects = send(
        &mut model,
        Event::PlayerFailed {
            session,
            message: "file missing".into(),
        },
    );

    assert!(player_ops(&effects).iter().any(|op| matches!(
        op,
        PlayerOperation::Load {
            media: MediaSource::Stream { .. },
            autoplay: true,
            ..
        }
    )));
    assert_eq!(model.downloading.as_deref(), Some("e1"));
    assert!(view_player(&model).error.is_none());
}

#[test]
fn cold_start_restores_the_active_episode_paused_and_loads_on_first_play() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(200);
    let mut model = model_with(vec![]);

    let effects = send(&mut model, saved_context(e));
    assert!(
        player_ops(&effects).is_empty(),
        "restore must not touch the engine"
    );
    let view = view_player(&model);
    assert!(!view.is_playing);
    assert_eq!(view.position_secs, 200);
    assert_eq!(view.feed_title, "Feed");

    let effects = send(&mut model, Event::TogglePlay);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load {
            start_secs: 197,
            autoplay: true,
            ..
        }]
    ));
}

#[test]
fn nothing_saved_means_no_player() {
    let mut model = model_with(vec![]);
    send(
        &mut model,
        Event::PlayContextLoaded(Box::new(StorageResult::NotFound)),
    );
    assert!(Pollux.view(&model).player.is_none());
}

#[test]
fn playing_saves_the_source_with_the_play_context() {
    let mut model = model_with(vec![episode("e1")]);

    let effects = send(&mut model, Event::PlayEpisode("e1".into()));

    assert!(storage_ops(&effects).iter().any(|op| matches!(
        op,
        StorageOperation::SavePlayContext { episode_id, source }
            if episode_id == "e1"
                && *source == EpisodeSource::Subscription { id: "sub".into() }
    )));
    assert_eq!(
        active(&model).source,
        EpisodeSource::Subscription { id: "sub".into() }
    );
}

#[test]
fn the_player_view_names_and_targets_the_source() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let view = view_player(&model);

    assert_eq!(
        view.source,
        EpisodeSource::Subscription { id: "sub".into() }
    );
    assert_eq!(view.source_title, "Feed");
}

#[test]
fn a_restored_episode_keeps_the_source_it_was_started_from() {
    let mut model = model_with(vec![]);
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(30);

    send(&mut model, saved_context(e));

    let view = view_player(&model);
    assert_eq!(
        view.source,
        EpisodeSource::Subscription { id: "sub".into() }
    );
    assert_eq!(view.source_title, "Feed");
}

#[test]
fn the_player_view_carries_the_show_notes_with_a_plain_text_fallback() {
    let mut e = episode("e1");
    e.description = Some("<p>Hello <b>world</b></p>".into());
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let view = view_player(&model);

    assert_eq!(
        view.description.as_deref(),
        Some("<p>Hello <b>world</b></p>")
    );
    assert_eq!(view.description_text.as_deref(), Some("Hello world"));
}

#[test]
fn an_episode_without_show_notes_has_none_to_show() {
    // Missing and blank descriptions both mean the show-notes page is left out.
    for description in [None, Some("   \n ".to_string())] {
        let mut e = episode("e1");
        e.description = description;
        let mut model = model_with(vec![e]);
        send(&mut model, Event::PlayEpisode("e1".into()));

        let view = view_player(&model);

        assert!(view.description.is_none());
        assert!(view.description_text.is_none());
    }
}
