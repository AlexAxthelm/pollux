//! The player reads its skip and rewind values from the settings, not from constants.

use super::tests::{active, episode, model_with, player_ops, send, tick, view_player};
use super::*;
use crate::settings::Settings;

fn with_settings(settings: Settings) -> Model {
    let mut model = model_with(vec![episode("e1")]);
    model.global_settings = settings;
    model
}

#[test]
fn skip_buttons_use_the_global_increments() {
    let mut model = with_settings(Settings {
        skip_forward_secs: Some(60),
        skip_back_secs: Some(10),
        ..Settings::default()
    });
    send(&mut model, Event::PlayEpisode("e1".into()));

    let effects = send(&mut model, Event::SkipForward);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { secs: 60 }]
    ));
    send(&mut model, Event::SkipBack);
    assert_eq!(active(&model).position_secs, 50);
}

#[test]
fn the_player_view_shows_the_resolved_increments() {
    let mut model = with_settings(Settings {
        skip_forward_secs: Some(45),
        skip_back_secs: Some(5),
        ..Settings::default()
    });
    send(&mut model, Event::PlayEpisode("e1".into()));
    let view = view_player(&model);
    assert_eq!(view.skip_forward_secs, 45);
    assert_eq!(view.skip_back_secs, 5);
}

#[test]
fn a_change_applies_to_the_next_skip_while_playing() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(&mut model, Event::SkipForward);
    assert_eq!(active(&model).position_secs, 30);

    model.global_settings.skip_forward_secs = Some(90);
    send(&mut model, Event::SkipForward);
    assert_eq!(active(&model).position_secs, 120);
    assert_eq!(view_player(&model).skip_forward_secs, 90);
}

#[test]
fn pause_then_play_rewinds_by_the_global_amount() {
    let mut model = with_settings(Settings {
        resume_rewind_secs: Some(10),
        ..Settings::default()
    });
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 50);
    send(&mut model, Event::Pause);
    send(&mut model, Event::Play);
    assert_eq!(active(&model).position_secs, 40);
}

#[test]
fn a_rewind_of_zero_turns_the_rewind_off() {
    let mut model = with_settings(Settings {
        resume_rewind_secs: Some(0),
        ..Settings::default()
    });
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 50);
    send(&mut model, Event::Pause);
    send(&mut model, Event::Play);
    assert_eq!(active(&model).position_secs, 50);
}

#[test]
fn starting_a_partly_played_episode_rewinds_by_the_global_amount() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(100);
    let mut model = model_with(vec![e]);
    model.global_settings.resume_rewind_secs = Some(20);

    let effects = send(&mut model, Event::PlayEpisode("e1".into()));
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load { start_secs: 80, .. }]
    ));
}
