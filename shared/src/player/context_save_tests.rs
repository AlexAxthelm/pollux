//! The saved play context is what a relaunch restores, so a failed save must be retried
//! rather than left to restore the wrong episode (or none). Position writes retry
//! themselves at the next checkpoint; the context is written once when playback starts,
//! so it needs its own retry. See the module docs in `player.rs`.

use super::tests::{
    active, checkpoints, episode, model_with, saved_context, send, storage_ops, tick,
};
use super::*;
use crate::defaults::CONTEXT_SAVE_ATTEMPTS;

fn context_saves(effects: &[Effect]) -> Vec<String> {
    storage_ops(effects)
        .into_iter()
        .filter_map(|op| match op {
            StorageOperation::SavePlayContext { episode_id, .. } => Some(episode_id),
            _ => None,
        })
        .collect()
}

fn saved(model: &mut Model, id: &str, result: StorageResult) {
    send(
        model,
        Event::PlayContextSaved {
            episode_id: id.into(),
            result: Box::new(result),
        },
    );
}

fn started() -> Model {
    let mut model = model_with(vec![episode("e1"), episode("e2")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    model
}

#[test]
fn a_failed_context_save_is_retried_at_the_next_checkpoint() {
    let mut model = started();
    saved(&mut model, "e1", StorageResult::Error("disk full".into()));

    // The next periodic checkpoint carries the retry along with the position.
    let effects = tick(&mut model, 30);

    assert_eq!(context_saves(&effects), vec!["e1".to_string()]);
    assert_eq!(checkpoints(&effects).len(), 1);
}

#[test]
fn the_retry_repeats_until_a_save_succeeds_and_then_stops() {
    let mut model = started();
    saved(&mut model, "e1", StorageResult::Error("disk full".into()));
    let effects = tick(&mut model, 30);
    assert_eq!(context_saves(&effects).len(), 1);

    // It fails again: the next checkpoint tries again.
    saved(&mut model, "e1", StorageResult::Error("disk full".into()));
    let effects = tick(&mut model, 60);
    assert_eq!(context_saves(&effects).len(), 1);

    saved(&mut model, "e1", StorageResult::Success);
    let effects = tick(&mut model, 90);
    assert!(context_saves(&effects).is_empty());
    assert_eq!(checkpoints(&effects).len(), 1, "positions still checkpoint");
}

#[test]
fn a_successful_save_is_not_repeated() {
    let mut model = started();
    saved(&mut model, "e1", StorageResult::Success);

    let effects = tick(&mut model, 30);

    assert!(context_saves(&effects).is_empty());
}

#[test]
fn a_failed_save_is_also_retried_when_pausing() {
    let mut model = started();
    saved(&mut model, "e1", StorageResult::Error("disk full".into()));

    let effects = send(&mut model, Event::Pause);

    assert_eq!(context_saves(&effects), vec!["e1".to_string()]);
}

#[test]
fn a_late_answer_for_a_replaced_episode_does_not_mark_the_new_one_saved() {
    let mut model = started();
    send(&mut model, Event::PlayEpisode("e2".into()));

    // e1's save resolves after e2 took over; e2's own is still outstanding.
    saved(&mut model, "e1", StorageResult::Success);

    assert!(!active(&model).context_saved);
}

#[test]
fn a_late_failure_for_a_replaced_episode_is_ignored() {
    let mut model = started();
    send(&mut model, Event::PlayEpisode("e2".into()));
    saved(&mut model, "e2", StorageResult::Success);

    saved(&mut model, "e1", StorageResult::Error("disk full".into()));

    let effects = tick(&mut model, 30);
    assert!(context_saves(&effects).is_empty());
}

#[test]
fn starting_another_episode_saves_its_context_afresh() {
    let mut model = started();
    saved(&mut model, "e1", StorageResult::Success);

    let effects = send(&mut model, Event::PlayEpisode("e2".into()));

    assert_eq!(context_saves(&effects), vec!["e2".to_string()]);
    assert!(!active(&model).context_saved);
}

fn fail_the_save(model: &mut Model) {
    let id = active(model).episode.id.clone();
    saved(model, &id, StorageResult::Error("disk full".into()));
}

/// Fails the save the playback start made, then lets every checkpoint retry and fail, until
/// the attempts are used up. Returns the attempts made, the first being the start's own.
fn exhaust_the_attempts(model: &mut Model) -> u32 {
    let mut attempts = 1;
    fail_the_save(model);
    let mut position = 0;
    while attempts < CONTEXT_SAVE_ATTEMPTS {
        position += 30;
        let effects = tick(model, position);
        assert_eq!(context_saves(&effects).len(), 1, "attempt {}", attempts + 1);
        attempts += 1;
        fail_the_save(model);
    }
    attempts
}

#[test]
fn a_save_that_keeps_failing_is_given_up_on() {
    let mut model = started();
    let attempts = exhaust_the_attempts(&mut model);
    assert_eq!(attempts, CONTEXT_SAVE_ATTEMPTS);

    // Further checkpoints no longer try, however long playback goes on.
    for position in [400, 430, 460] {
        let effects = tick(&mut model, position);
        assert!(context_saves(&effects).is_empty());
    }
    assert!(!active(&model).context_saved);
}

#[test]
fn pausing_or_backgrounding_does_not_retry_a_given_up_save_either() {
    let mut model = started();
    exhaust_the_attempts(&mut model);

    assert!(context_saves(&send(&mut model, Event::AppBackgrounded)).is_empty());
    assert!(context_saves(&send(&mut model, Event::Pause)).is_empty());
}

#[test]
fn giving_up_on_one_episode_does_not_stop_the_next_from_saving() {
    let mut model = started();
    exhaust_the_attempts(&mut model);

    let effects = send(&mut model, Event::PlayEpisode("e2".into()));

    assert_eq!(context_saves(&effects), vec!["e2".to_string()]);
    fail_the_save(&mut model);
    let effects = tick(&mut model, 30);
    assert_eq!(context_saves(&effects), vec!["e2".to_string()]);
}

#[test]
fn a_save_that_lands_before_the_limit_stops_the_retries() {
    let mut model = started();
    fail_the_save(&mut model);
    tick(&mut model, 30);
    saved(&mut model, "e1", StorageResult::Success);

    for position in [60, 90, 120] {
        assert!(context_saves(&tick(&mut model, position)).is_empty());
    }
}

#[test]
fn a_restored_episode_is_already_saved() {
    let mut model = model_with(vec![]);
    send(&mut model, saved_context(episode("e1")));
    send(&mut model, Event::TogglePlay);

    let effects = tick(&mut model, 30);

    assert!(context_saves(&effects).is_empty());
}
