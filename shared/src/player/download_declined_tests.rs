//! Playback downloads what it streams, but not against the listener's wishes: once they
//! cancel or delete an episode's download, resuming, restarting or recovering from a lost
//! file must not bring it back. Only asking for the download again does. See the module
//! docs in `player.rs`.

use super::tests::{episode, model_with, player_ops, send, session};
use super::*;
use crate::capabilities::download::{DownloadOperation, DownloadResult};

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

/// Streams "e1" (which starts its download), then the listener cancels that download.
fn streaming_with_the_download_cancelled() -> Model {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    assert_eq!(model.downloading.as_deref(), Some("e1"));
    send(&mut model, Event::CancelDownload("e1".into()));
    // The shell has cancelled the task and says so.
    send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Cancelled),
        },
    );
    assert!(model.downloading.is_none());
    model
}

#[test]
fn resuming_does_not_restart_a_download_the_listener_cancelled() {
    let mut model = streaming_with_the_download_cancelled();

    send(&mut model, Event::TogglePlay); // pause
    let effects = send(&mut model, Event::TogglePlay); // play

    assert_eq!(downloads_started(&effects), 0);
    assert!(model.downloading.is_none());
}

#[test]
fn starting_the_episode_again_does_not_restart_it_either() {
    let mut model = streaming_with_the_download_cancelled();
    send(&mut model, Event::PlayEpisode("e2".into()));

    let effects = send(&mut model, Event::PlayEpisode("e1".into()));

    // e2's download is the one running; e1's must not be waiting behind it.
    assert_eq!(downloads_started(&effects), 0);
    assert!(model.download_queue.is_empty());
    assert_eq!(
        model
            .episodes
            .iter()
            .find(|e| e.id == "e1")
            .unwrap()
            .download_status,
        DownloadStatus::NotDownloaded
    );
}

#[test]
fn a_cancelled_queued_download_is_respected_too() {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    // e2's download is running, so e1's waits in the queue.
    send(&mut model, Event::DownloadEpisode("e2".into()));
    send(&mut model, Event::PlayEpisode("e1".into()));
    assert_eq!(model.download_queue.len(), 1);
    send(&mut model, Event::CancelDownload("e1".into()));
    assert!(model.download_queue.is_empty());

    send(&mut model, Event::TogglePlay);
    send(&mut model, Event::TogglePlay);

    assert!(model.download_queue.is_empty());
}

#[test]
fn losing_a_file_the_listener_deleted_does_not_bring_it_back() {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/e1.mp3".into());
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    // They delete the download while it plays; the engine then loses the file.
    send(&mut model, Event::DeleteDownload("e1".into()));
    let session = session(&model);
    let effects = send(
        &mut model,
        Event::PlayerMediaUnusable {
            session,
            message: "file gone".into(),
        },
    );

    assert_eq!(downloads_started(&effects), 0);
    assert!(model.downloading.is_none());
    assert!(
        player_ops(&effects).iter().any(|op| matches!(
            op,
            PlayerOperation::Load {
                media: MediaSource::Stream { .. },
                ..
            }
        )),
        "playback still carries on from the stream"
    );
}

#[test]
fn asking_for_the_download_again_is_honoured() {
    let mut model = streaming_with_the_download_cancelled();

    send(&mut model, Event::DownloadEpisode("e1".into()));

    assert_eq!(model.downloading.as_deref(), Some("e1"));
}

#[test]
fn after_asking_again_playback_may_retry_it_as_usual() {
    let mut model = streaming_with_the_download_cancelled();
    send(&mut model, Event::DownloadEpisode("e1".into()));
    // That download fails; a later resume tries again, as for any failed download.
    send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(DownloadResult::Error("offline".into())),
        },
    );

    send(&mut model, Event::TogglePlay);
    send(&mut model, Event::TogglePlay);

    assert_eq!(model.downloading.as_deref(), Some("e1"));
}

#[test]
fn cancelling_one_episode_does_not_stop_another_from_downloading() {
    let mut model = streaming_with_the_download_cancelled();

    let effects = send(&mut model, Event::PlayEpisode("e2".into()));

    assert_eq!(downloads_started(&effects), 1);
    assert_eq!(model.downloading.as_deref(), Some("e2"));
}
