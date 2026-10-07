//! A downloaded file the engine can't use is replaced by a fresh download once. If the fresh
//! one is unusable too, the file is bad (not just missing), and downloading it again would
//! loop for as long as the listener keeps playing. See the module docs in `player.rs`.

use super::tests::{active, episode, model_with, player_ops, send, session};
use super::*;
use crate::capabilities::download::{DownloadOperation, DownloadResult};

fn downloaded_episode() -> Episode {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/e1.mp3".into());
    e
}

fn unusable(model: &mut Model) -> Vec<Effect> {
    let session = session(model);
    send(
        model,
        Event::PlayerMediaUnusable {
            session,
            message: "can't decode".into(),
        },
    )
}

fn download_finished(model: &mut Model) -> Vec<Effect> {
    send(
        model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Completed {
                local_path: "Downloads/e1.mp3".into(),
                size_bytes: 1000,
            }),
        },
    )
}

fn downloads_started(effects: &[Effect]) -> usize {
    effects
        .iter()
        .filter(|e| {
            matches!(
                e,
                Effect::Download(r) if matches!(r.operation, DownloadOperation::Download { .. })
            )
        })
        .count()
}

fn loads_stream(effects: &[Effect]) -> bool {
    player_ops(effects).iter().any(|op| {
        matches!(
            op,
            PlayerOperation::Load {
                media: MediaSource::Stream { .. },
                ..
            }
        )
    })
}

/// Plays a downloaded episode whose file is bad, through the first fallback and the
/// re-download, up to the swap back onto the (still bad) fresh file.
fn up_to_the_second_failure() -> (Model, usize) {
    let mut model = model_with(vec![downloaded_episode()]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let first = unusable(&mut model);
    let mut downloads = downloads_started(&first);
    assert_eq!(
        downloads, 1,
        "a bad file is replaced by a fresh download once"
    );

    let finished = download_finished(&mut model);
    downloads += downloads_started(&finished);
    assert!(
        player_ops(&finished).iter().any(|op| matches!(
            op,
            PlayerOperation::Load {
                media: MediaSource::Local { .. },
                ..
            }
        )),
        "the fresh file is tried"
    );
    (model, downloads)
}

#[test]
fn a_fresh_file_that_is_also_unusable_is_not_downloaded_again() {
    let (mut model, downloads) = up_to_the_second_failure();

    let second = unusable(&mut model);

    assert_eq!(downloads_started(&second), 0);
    assert_eq!(downloads, 1);
    assert!(model.downloading.is_none());
    assert!(model.download_queue.is_empty());
    assert!(loads_stream(&second), "playback carries on from the stream");
}

#[test]
fn the_failed_download_says_why() {
    let (mut model, _) = up_to_the_second_failure();

    let effects = unusable(&mut model);

    assert_eq!(
        active(&model).episode.download_status,
        DownloadStatus::Failed
    );
    assert!(model.download_errors.contains_key("e1"));
    assert!(super::tests::storage_ops(&effects)
        .iter()
        .any(|op| matches!(
            op,
            StorageOperation::UpdateDownloadState {
                status: DownloadStatus::Failed,
                ..
            }
        )));
}

#[test]
fn pressing_play_does_not_retry_a_file_that_proved_bad() {
    let (mut model, _) = up_to_the_second_failure();
    unusable(&mut model);

    send(&mut model, Event::TogglePlay);
    let effects = send(&mut model, Event::TogglePlay);

    assert_eq!(downloads_started(&effects), 0);
    assert!(model.downloading.is_none());
}

#[test]
fn a_file_that_plays_clears_the_suspicion() {
    let mut model = model_with(vec![downloaded_episode()]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    unusable(&mut model);
    download_finished(&mut model);

    // The fresh file loads and the engine reports its duration: it is good.
    let session = session(&model);
    send(
        &mut model,
        Event::PlayerDuration {
            session,
            duration_secs: 600,
        },
    );

    // Deleted behind our back some day later: a missing file again, replaced once more.
    let effects = unusable(&mut model);
    assert_eq!(downloads_started(&effects), 1);
}

#[test]
fn a_stream_that_loads_does_not_clear_the_suspicion() {
    let mut model = model_with(vec![downloaded_episode()]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    unusable(&mut model);

    // The stream is fine, but says nothing about the file.
    let session = session(&model);
    send(
        &mut model,
        Event::PlayerDuration {
            session,
            duration_secs: 600,
        },
    );
    download_finished(&mut model);
    let effects = unusable(&mut model);

    assert_eq!(downloads_started(&effects), 0);
}

#[test]
fn a_missing_file_is_still_replaced_the_first_time() {
    let mut model = model_with(vec![downloaded_episode()]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    let effects = unusable(&mut model);

    assert_eq!(downloads_started(&effects), 1);
    assert!(loads_stream(&effects));
}
