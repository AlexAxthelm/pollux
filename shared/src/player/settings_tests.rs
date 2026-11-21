//! The player reads its skip and rewind values from the settings (the episode's
//! subscription's, then the global ones), not from constants.

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

// -- A subscription's own settings sit between the episode and the global ones (ADR-0044) --

fn with_levels(global: Settings, subscription: Settings) -> Model {
    let mut model = with_settings(global);
    model
        .subscription_settings
        .insert("sub".to_string(), subscription);
    model
}

#[test]
fn a_subscription_setting_beats_the_global_one() {
    let mut model = with_levels(
        Settings {
            skip_forward_secs: Some(45),
            ..Settings::default()
        },
        Settings {
            skip_forward_secs: Some(60),
            ..Settings::default()
        },
    );
    send(&mut model, Event::PlayEpisode("e1".into()));
    let effects = send(&mut model, Event::SkipForward);
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Seek { secs: 60 }]
    ));
    assert_eq!(view_player(&model).skip_forward_secs, 60);
}

#[test]
fn what_a_subscription_leaves_unset_falls_through_to_global() {
    let mut model = with_levels(
        Settings {
            skip_forward_secs: Some(45),
            skip_back_secs: Some(20),
            ..Settings::default()
        },
        Settings {
            skip_back_secs: Some(5),
            ..Settings::default()
        },
    );
    send(&mut model, Event::PlayEpisode("e1".into()));
    let view = view_player(&model);
    assert_eq!(view.skip_forward_secs, 45);
    assert_eq!(view.skip_back_secs, 5);
}

#[test]
fn another_subscriptions_settings_do_not_apply() {
    let mut model = with_settings(Settings::default());
    model.subscription_settings.insert(
        "other".to_string(),
        Settings {
            skip_forward_secs: Some(90),
            ..Settings::default()
        },
    );
    send(&mut model, Event::PlayEpisode("e1".into()));
    assert_eq!(
        view_player(&model).skip_forward_secs,
        crate::defaults::SKIP_FORWARD_SECS
    );
}

#[test]
fn zero_in_a_subscription_beats_a_nonzero_global() {
    let mut model = with_levels(
        Settings {
            resume_rewind_secs: Some(10),
            ..Settings::default()
        },
        Settings {
            resume_rewind_secs: Some(0),
            ..Settings::default()
        },
    );
    send(&mut model, Event::PlayEpisode("e1".into()));
    tick(&mut model, 50);
    send(&mut model, Event::Pause);
    send(&mut model, Event::Play);
    assert_eq!(active(&model).position_secs, 50);
}

#[test]
fn starting_a_partly_played_episode_rewinds_by_its_subscriptions_amount() {
    let mut e = episode("e1");
    e.playback_status = PlaybackStatus::InProgress;
    e.playback_position_secs = Some(100);
    let mut model = model_with(vec![e]);
    model.global_settings.resume_rewind_secs = Some(20);
    model
        .subscription_settings
        .entry("sub".to_string())
        .or_default()
        .resume_rewind_secs = Some(7);

    let effects = send(&mut model, Event::PlayEpisode("e1".into()));
    assert!(matches!(
        player_ops(&effects).as_slice(),
        [PlayerOperation::Load { start_secs: 93, .. }]
    ));
}

#[test]
fn changing_and_resetting_a_subscription_setting_applies_to_the_next_skip() {
    let mut model = model_with(vec![episode("e1")]);
    send(&mut model, Event::PlayEpisode("e1".into()));
    send(
        &mut model,
        Event::SettingsLoaded(Box::new(crate::StorageResult::AllSettings(vec![]))),
    );
    send(
        &mut model,
        Event::SetSubscriptionSetting {
            subscription_id: "sub".into(),
            change: crate::SettingChange::SkipForwardSecs(Some(60)),
        },
    );
    send(&mut model, Event::SkipForward);
    assert_eq!(active(&model).position_secs, 60);
    assert_eq!(view_player(&model).skip_forward_secs, 60);

    send(
        &mut model,
        Event::SetSubscriptionSetting {
            subscription_id: "sub".into(),
            change: crate::SettingChange::SkipForwardSecs(None),
        },
    );
    send(&mut model, Event::SkipForward);
    assert_eq!(active(&model).position_secs, 90);
    assert_eq!(view_player(&model).skip_forward_secs, 30);
}
