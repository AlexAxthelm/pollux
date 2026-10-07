//! An interruption's end may only restart audio that the interruption itself stopped. Once
//! the listener (or a route change) has paused it, nothing may resume it behind their back.
//! See the module docs in `player.rs`.

use super::tests::{episode, model_with, player_ops, send};
use super::*;

fn playing() -> Model {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    model
}

fn call_ends(model: &mut Model) -> Vec<PlayerOperation> {
    player_ops(&send(
        model,
        Event::InterruptionEnded {
            should_resume: true,
        },
    ))
}

#[test]
fn a_pause_during_the_interruption_cancels_the_auto_resume() {
    let mut model = playing();
    send(&mut model, Event::Interrupted { resumable: true });
    // Already paused by the interruption; the listener also presses pause (a lock-screen
    // or headphone command that reached us before the call ended).
    send(&mut model, Event::Pause);

    assert!(
        call_ends(&mut model).is_empty(),
        "the listener paused: the call ending must not restart audio"
    );
    assert!(!model.active_playback.as_ref().unwrap().is_playing);
}

#[test]
fn unplugging_headphones_during_the_interruption_cancels_the_auto_resume() {
    let mut model = playing();
    send(&mut model, Event::Interrupted { resumable: true });
    // The headphones come out while the call is up: a route change, never resumable.
    send(&mut model, Event::Interrupted { resumable: false });

    assert!(
        call_ends(&mut model).is_empty(),
        "the output went away: resuming could play through the speaker"
    );
}

#[test]
fn a_second_resumable_interruption_keeps_the_resume_armed() {
    let mut model = playing();
    send(&mut model, Event::Interrupted { resumable: true });
    // Siri during the call, say: still the interruption that paused it.
    send(&mut model, Event::Interrupted { resumable: true });

    assert!(!call_ends(&mut model).is_empty());
}

#[test]
fn an_interruption_still_resumes_playback_it_paused() {
    let mut model = playing();
    send(&mut model, Event::Interrupted { resumable: true });

    assert!(call_ends(&mut model)
        .iter()
        .any(|op| matches!(op, PlayerOperation::Seek { .. } | PlayerOperation::Play)));
    assert!(model.active_playback.as_ref().unwrap().is_playing);
}

#[test]
fn pausing_by_hand_when_nothing_was_interrupted_changes_nothing_else() {
    let mut model = playing();
    send(&mut model, Event::Pause);

    assert!(call_ends(&mut model).is_empty());
}
