//! The saved play context is what a relaunch restores, so a failed save must be retried
//! rather than left to restore the wrong episode (or none). Position writes retry
//! themselves at the next checkpoint; the context is written once when playback starts,
//! so it needs its own retry. See the module docs in `player.rs`.

use super::tests::{
    active, checkpoints, episode, model_with, saved_context, send, storage_ops, tick,
};
use super::*;

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

#[test]
fn a_restored_episode_is_already_saved() {
    let mut model = model_with(vec![]);
    send(&mut model, saved_context(episode("e1")));
    send(&mut model, Event::TogglePlay);

    let effects = tick(&mut model, 30);

    assert!(context_saves(&effects).is_empty());
}
