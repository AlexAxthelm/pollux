//! Downloads the player starts or retries for the episode it has active, which must not
//! depend on that episode's feed being the one on screen: after a cold start the list is
//! empty until the user opens a feed. See the module docs in `player.rs`.

use super::tests::{active, episode, model_with, player_ops, saved_context, send, session};
use super::*;
use crate::capabilities::download::DownloadResult;

/// A cold start: the saved episode is restored paused, and no feed has been opened.
fn cold_started(configure: impl FnOnce(&mut Episode)) -> Model {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(200);
    configure(&mut e);
    let mut model = model_with(vec![]);
    send(&mut model, saved_context(e));
    assert!(
        model.episodes.is_empty(),
        "no feed is open after a cold start"
    );
    model
}

fn download_state_writes(effects: &[Effect]) -> Vec<DownloadStatus> {
    super::tests::storage_ops(effects)
        .into_iter()
        .filter_map(|op| match op {
            StorageOperation::UpdateDownloadState { status, .. } => Some(status),
            _ => None,
        })
        .collect()
}

#[test]
fn a_missing_file_after_a_cold_start_still_redownloads() {
    let mut model = cold_started(|e| {
        e.download_status = DownloadStatus::Downloaded;
        e.local_path = Some("Downloads/gone.mp3".into());
    });
    send(&mut model, Event::TogglePlay);

    let session = session(&model);
    let effects = send(
        &mut model,
        Event::PlayerMediaUnusable {
            session,
            message: "file missing".into(),
        },
    );

    assert!(player_ops(&effects).iter().any(|op| matches!(
        op,
        PlayerOperation::Load {
            media: MediaSource::Stream { .. },
            ..
        }
    )));
    assert_eq!(model.downloading.as_deref(), Some("e1"));
    assert_eq!(
        download_state_writes(&effects),
        vec![
            DownloadStatus::NotDownloaded,
            DownloadStatus::Queued,
            DownloadStatus::Downloading
        ]
    );
}

#[test]
fn resuming_a_restored_episode_whose_download_failed_retries_it() {
    let mut model = cold_started(|e| e.download_status = DownloadStatus::Failed);

    let effects = send(&mut model, Event::TogglePlay);

    assert_eq!(model.downloading.as_deref(), Some("e1"));
    assert_eq!(
        download_state_writes(&effects),
        vec![DownloadStatus::Queued, DownloadStatus::Downloading]
    );
    assert!(
        player_ops(&effects)
            .iter()
            .any(|op| matches!(op, PlayerOperation::Load { autoplay: true, .. })),
        "the retry must not get in the way of playing"
    );
}

#[test]
fn resuming_a_restored_episode_that_was_never_downloaded_starts_the_download() {
    let mut model = cold_started(|_| {});

    send(&mut model, Event::TogglePlay);

    assert_eq!(model.downloading.as_deref(), Some("e1"));
}

#[test]
fn resuming_does_not_redownload_a_file_that_is_there() {
    let mut model = cold_started(|e| {
        e.download_status = DownloadStatus::Downloaded;
        e.local_path = Some("Downloads/e1.mp3".into());
    });

    let effects = send(&mut model, Event::TogglePlay);

    assert!(model.downloading.is_none());
    assert!(download_state_writes(&effects).is_empty());
}

#[test]
fn resuming_does_not_enqueue_a_download_that_is_already_underway() {
    let mut model = cold_started(|_| {});
    send(&mut model, Event::TogglePlay);
    assert_eq!(model.downloading.as_deref(), Some("e1"));

    // Pause and resume while it downloads: still one download, not two.
    send(&mut model, Event::TogglePlay);
    let effects = send(&mut model, Event::TogglePlay);

    assert!(download_state_writes(&effects).is_empty());
    assert!(model.download_queue.is_empty());
}

#[test]
fn a_download_started_for_a_restored_episode_swaps_playback_onto_the_file() {
    let mut model = cold_started(|_| {});
    send(&mut model, Event::TogglePlay);

    let effects = send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Completed {
                local_path: "Downloads/e1.mp3".into(),
                size_bytes: 1234,
            }),
        },
    );

    assert_eq!(
        active(&model).episode.download_status,
        DownloadStatus::Downloaded
    );
    assert!(player_ops(&effects).iter().any(|op| matches!(
        op,
        PlayerOperation::Load {
            media: MediaSource::Local { .. },
            ..
        }
    )));
}

#[test]
fn a_failed_download_of_the_active_episode_is_recorded_so_a_resume_can_retry() {
    let mut model = cold_started(|_| {});
    send(&mut model, Event::TogglePlay);

    send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Error("offline".into())),
        },
    );
    assert_eq!(
        active(&model).episode.download_status,
        DownloadStatus::Failed
    );

    // Pause, then play again: the failed download is retried.
    send(&mut model, Event::TogglePlay);
    send(&mut model, Event::TogglePlay);
    assert_eq!(model.downloading.as_deref(), Some("e1"));
}
