//! When an episode counts as played, what restore brings back, and which engine news is
//! stale. See the module docs in `player.rs`.

use crux_core::App;

use super::tests::{
    active, checkpoints, episode, model_with, player_ops, saved_context, send, send_ended, session,
    storage_ops, tick, view_player,
};
use super::*;
use crate::Pollux;

#[test]
fn backgrounding_in_the_tail_while_listening_does_not_finish_the_episode() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 590);

    let effects = send(&mut model, Event::AppBackgrounded);

    // Still listening to the last seconds: only the place is saved, never "played".
    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::InProgress, Some(590))]
    );
    assert!(model.active_playback.is_some());
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );
}

#[test]
fn a_periodic_checkpoint_in_the_tail_never_resets_the_position() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 585);

    // Crosses the 10s checkpoint interval inside the tail.
    let effects = tick(&mut model, 596);

    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::InProgress, Some(596))]
    );
    assert_eq!(active(&model).position_secs, 596);
}

#[test]
fn a_played_episode_is_not_restored_and_its_saved_context_is_cleared() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::Played;
    e.playback_position_secs = Some(0);
    let mut model = model_with(vec![]);

    let effects = send(&mut model, saved_context(e));

    assert!(Pollux.view(&model).player.is_none());
    assert!(storage_ops(&effects)
        .iter()
        .any(|op| matches!(op, StorageOperation::ClearPlayContext)));
}

#[test]
fn an_in_progress_episode_is_still_restored() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(595);
    let mut model = model_with(vec![]);

    send(&mut model, saved_context(e));

    assert_eq!(view_player(&model).position_secs, 595);
}

#[test]
fn pausing_in_the_tail_finishes_the_episode_and_clears_the_player() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 590);

    let effects = send(&mut model, Event::Pause);

    assert!(model.active_playback.is_none());
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::Played, Some(0))]
    );
    assert!(storage_ops(&effects)
        .iter()
        .any(|op| matches!(op, StorageOperation::ClearPlayContext)));
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Stop]
    ));
}

#[test]
fn pausing_before_the_tail_just_saves_the_place() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 300);

    let effects = send(&mut model, Event::Pause);

    assert!(model.active_playback.is_some());
    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::InProgress, Some(300))]
    );
}

#[test]
fn a_system_interruption_in_the_tail_pauses_without_finishing() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 590);

    let effects = send(&mut model, Event::Interrupted { resumable: true });

    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Pause]
    ));
    assert_eq!(
        checkpoints(&effects),
        vec![(PlaybackStatus::InProgress, Some(590))]
    );
    assert!(!view_player(&model).is_playing);

    // The call ended and the system says it's fine to resume: the last seconds play out.
    let effects = send(
        &mut model,
        Event::InterruptionEnded {
            should_resume: true,
        },
    );
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { .. }, PlayerOperation::Play]
    ));
}

#[test]
fn switching_episode_from_the_tail_marks_the_outgoing_one_played() {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 595);

    let effects = send(&mut model, Event::PlayEpisode("e2".into()));

    assert!(storage_ops(&effects).iter().any(|op| matches!(
        op,
        StorageOperation::UpdatePlaybackStatus {
            episode_id,
            status: PlaybackStatus::Played,
            position_secs: Some(0),
        } if episode_id == "e1"
    )));
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
    assert_eq!(active(&model).episode.id, "e2");
}

#[test]
fn switching_episode_before_the_tail_keeps_the_place() {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 120);

    let effects = send(&mut model, Event::PlayEpisode("e2".into()));

    assert!(storage_ops(&effects).iter().any(|op| matches!(
        op,
        StorageOperation::UpdatePlaybackStatus {
            episode_id,
            status: PlaybackStatus::InProgress,
            position_secs: Some(120),
        } if episode_id == "e1"
    )));
}

#[test]
fn a_short_episode_is_not_finished_by_pausing_near_its_start() {
    let mut e = episode("e1");
    e.duration_secs = Some(10);
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 3);

    send(&mut model, Event::Pause);

    assert!(
        model.active_playback.is_some(),
        "3s into a 10s episode is not the tail"
    );
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );

    // It still finishes when it actually ends.
    send(&mut model, Event::Play);
    send_ended(&mut model);
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
}

#[test]
fn an_unknown_duration_never_finishes_by_pausing() {
    let mut e = episode("e1");
    e.duration_secs = None;
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 5000);

    send(&mut model, Event::Pause);

    assert!(model.active_playback.is_some());
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );

    // Only the engine reaching the end finishes it.
    send(&mut model, Event::Play);
    send_ended(&mut model);
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
}

#[test]
fn the_engines_duration_beats_a_too_short_feed_duration() {
    // The feed claims 600s; the file is really an hour long.
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let session = session(&model);
    send(
        &mut model,
        Event::PlayerDuration {
            session,
            duration_secs: 3600,
        },
    );
    tick(&mut model, 590);

    send(&mut model, Event::Pause);

    assert!(
        model.active_playback.is_some(),
        "590s of 3600s is nowhere near the end"
    );
    assert_eq!(
        model.episodes[0].playback_status,
        PlaybackStatus::InProgress
    );
}

#[test]
fn engine_events_from_a_replaced_load_are_ignored() {
    let mut e = episode("e1");
    e.download_status = DownloadStatus::Downloaded;
    e.local_path = Some("Downloads/gone.mp3".into());
    let mut model = model_with(vec![e]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let first = session(&model);

    // The local file fails, so playback is re-loaded from the network: a new session.
    send(
        &mut model,
        Event::PlayerFailed {
            session: first,
            message: "file missing".into(),
        },
    );
    assert_ne!(session(&model), first);

    // Late news from the first load must not touch the second.
    send(
        &mut model,
        Event::PlayerTick {
            session: first,
            position_secs: 99,
        },
    );
    send(&mut model, Event::PlayerEnded { session: first });
    assert_eq!(active(&model).position_secs, 0);
    assert!(model.active_playback.is_some());
}

#[test]
fn a_source_swap_starts_a_new_session_and_drops_the_old_loads_news() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let streaming = session(&model);

    send(
        &mut model,
        Event::DownloadFinished {
            episode_id: "e1".into(),
            result: Box::new(crate::capabilities::download::DownloadResult::Completed {
                local_path: "Downloads/x.mp3".into(),
                size_bytes: 10,
            }),
        },
    );
    assert_ne!(session(&model), streaming);

    // The streaming item's end-of-file arriving late must not finish the episode.
    send(&mut model, Event::PlayerEnded { session: streaming });
    assert!(model.active_playback.is_some());
}

#[test]
fn news_for_a_finished_episode_is_ignored() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    let finished = session(&model);
    send_ended(&mut model);
    assert!(model.active_playback.is_none());

    // A straggling tick or second end event after completion is a no-op.
    let effects = send(
        &mut model,
        Event::PlayerTick {
            session: finished,
            position_secs: 599,
        },
    );
    assert!(effects.is_empty());
    assert!(model.active_playback.is_none());
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
}

#[test]
fn seeking_to_the_very_end_while_paused_finishes_the_episode() {
    // The engine only reports the end of the file while it is playing, so a skip or
    // scrub that lands on the end while paused would otherwise leave the episode
    // stuck active at 100%.
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, Event::Interrupted { resumable: true });
    assert!(!view_player(&model).is_playing);

    let effects = send(&mut model, Event::SeekTo(600));

    assert!(model.active_playback.is_none());
    assert_eq!(model.episodes[0].playback_status, PlaybackStatus::Played);
    assert!(storage_ops(&effects)
        .iter()
        .any(|op| matches!(op, StorageOperation::ClearPlayContext)));
}

#[test]
fn a_seek_that_lands_short_of_the_end_does_not_finish() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));

    send(&mut model, Event::SeekTo(599));

    assert_eq!(active(&model).position_secs, 599);
}

// --- Interruptions ---------------------------------------------------------------------

fn interrupted() -> Event {
    Event::Interrupted { resumable: true }
}

fn interruption_ended() -> Event {
    Event::InterruptionEnded {
        should_resume: true,
    }
}

#[test]
fn an_interruption_that_paused_playback_resumes_when_it_ends() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 100);
    send(&mut model, interrupted());
    assert!(!view_player(&model).is_playing);

    let effects = send(&mut model, interruption_ended());

    assert!(view_player(&model).is_playing);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { secs: 97 }, PlayerOperation::Play]
    ));
}

#[test]
fn an_interruption_does_not_resume_playback_the_listener_had_paused() {
    // The system reports "should resume" after any interruption, including one that found
    // playback already paused. The pause was the listener's choice and stands.
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 100);
    send(&mut model, Event::Pause);

    send(&mut model, interrupted());
    let effects = send(&mut model, interruption_ended());

    assert!(!view_player(&model).is_playing);
    assert!(player_ops(&effects).is_empty());
}

#[test]
fn an_interruption_that_says_not_to_resume_leaves_playback_paused() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, interrupted());

    let effects = send(
        &mut model,
        Event::InterruptionEnded {
            should_resume: false,
        },
    );

    assert!(!view_player(&model).is_playing);
    assert!(player_ops(&effects).is_empty());
}

#[test]
fn a_route_change_pause_is_never_resumed_by_a_later_interruption() {
    // Unplugged headphones pause playback but have no matching "ended" event. The arming
    // must not linger and let an unrelated call, hours later, restart the audio.
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, Event::Interrupted { resumable: false });
    assert!(!view_player(&model).is_playing);

    // Later: a call begins and ends while playback is paused.
    send(&mut model, interrupted());
    let effects = send(&mut model, interruption_ended());

    assert!(!view_player(&model).is_playing);
    assert!(player_ops(&effects).is_empty());
}

#[test]
fn the_listener_taking_over_during_an_interruption_cancels_the_auto_resume() {
    // Interrupted, then the listener plays and pauses by hand while the interruption is
    // still nominally open: its end must not start playback again.
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, interrupted());
    send(&mut model, Event::Play);
    send(&mut model, Event::Pause);
    assert!(!view_player(&model).is_playing);

    let effects = send(&mut model, interruption_ended());

    assert!(!view_player(&model).is_playing);
    assert!(player_ops(&effects).is_empty());
}

#[test]
fn scrubbing_during_an_interruption_cancels_the_auto_resume() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, interrupted());
    send(&mut model, Event::SeekTo(300));

    send(&mut model, interruption_ended());

    assert!(!view_player(&model).is_playing);
}

#[test]
fn the_end_of_an_interruption_with_nothing_active_does_nothing() {
    let mut model = model_with(vec![]);

    let effects = send(&mut model, interruption_ended());

    assert!(effects.is_empty());
}
