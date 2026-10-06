//! Player policy. The shell owns the audio engine (see `capabilities::player`); this
//! module decides what it should do: which source to load, where to start, how far a
//! skip goes, when position is persisted, and when an episode counts as played.
//!
//! Engine-originated news arrives as ordinary events (`PlayerTick`, `PlayerEnded`, …)
//! rather than request results, mirroring how download progress is reported.

use crux_core::{render::render, Command};

use crate::app::{enqueue_download, reset_to_not_downloaded, Event};
use crate::capabilities::player::{PlayerOperation, PlayerResult, PlayerSource};
use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::defaults::{
    PLAYED_TOLERANCE_SECS, POSITION_CHECKPOINT_SECS, RESUME_REWIND_SECS, SKIP_BACKWARD_SECS,
    SKIP_FORWARD_SECS,
};
use crate::domain::{DownloadStatus, Episode, PlaybackStatus};
use crate::effect::Effect;
use crate::model::{ActivePlayback, Model};
use crate::view_model::PlayerView;

#[cfg(test)]
mod tests;

type Cmd = Command<Effect, Event>;

/// Where an episode should play from: its downloaded file when we have one, else the
/// network.
fn source_for(episode: &Episode) -> PlayerSource {
    match (&episode.download_status, &episode.local_path) {
        (DownloadStatus::Downloaded, Some(path)) => PlayerSource::Local {
            local_path: path.clone(),
        },
        _ => PlayerSource::Stream {
            url: episode.enclosure_url.clone(),
        },
    }
}

fn player_op(episode_id: &str, op: PlayerOperation) -> Cmd {
    let id = episode_id.to_string();
    Command::request_from_shell(op).then_send(move |r| Event::PlayerResponded {
        episode_id: id.clone(),
        result: Box::new(r),
    })
}

/// Persists an episode's playback state. Best-effort: a failed write only costs
/// durability (the next checkpoint retries), so the result is dropped.
fn persist_playback(episode_id: &str, status: PlaybackStatus, position_secs: Option<u32>) -> Cmd {
    Command::request_from_shell(StorageOperation::UpdatePlaybackStatus {
        episode_id: episode_id.to_string(),
        status,
        position_secs,
    })
    .then_send(|r| Event::PlaybackPersisted(Box::new(r)))
}

/// Mirrors a playback-state change onto the feed on screen, if that episode is loaded,
/// so its row updates without a reload (see the note at `SelectSubscription`).
fn sync_episode_row(
    model: &mut Model,
    episode_id: &str,
    status: &PlaybackStatus,
    position_secs: Option<u32>,
) {
    if let Some(row) = model.episodes.iter_mut().find(|e| e.id == episode_id) {
        row.playback_status = status.clone();
        row.playback_position_secs = position_secs;
    }
}

/// The status and stored position for the current playhead: within the played
/// tolerance of the end means played (position reset), otherwise in-progress.
fn checkpoint_state(active: &ActivePlayback) -> (PlaybackStatus, Option<u32>) {
    let duration = active.duration_secs.or(active.episode.duration_secs);
    match duration {
        Some(d)
            if active.position_secs > 0
                && active.position_secs.saturating_add(PLAYED_TOLERANCE_SECS) >= d =>
        {
            (PlaybackStatus::Played, Some(0))
        }
        _ => (PlaybackStatus::InProgress, Some(active.position_secs)),
    }
}

/// Persists the current position/status. Called on pause, seek, backgrounding and
/// every `POSITION_CHECKPOINT_SECS` of playback.
fn checkpoint(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    let (status, position) = checkpoint_state(active);
    active.last_checkpoint_secs = active.position_secs;
    active.episode.playback_status = status.clone();
    active.episode.playback_position_secs = position;
    let id = active.episode.id.clone();
    sync_episode_row(model, &id, &status, position);
    persist_playback(&id, status, position)
}

/// Starts playing `episode_id` from the feed on screen. Resumes in place if it is
/// already the active episode.
pub(crate) fn play_episode(model: &mut Model, episode_id: &str) -> Cmd {
    if model
        .active_playback
        .as_ref()
        .is_some_and(|a| a.episode.id == episode_id)
    {
        return play(model);
    }
    let Some(episode) = model.episodes.iter().find(|e| e.id == episode_id).cloned() else {
        return render();
    };

    // Whatever was playing keeps its place before it's replaced.
    let outgoing = checkpoint(model);

    let source = source_for(&episode);
    // A played episode restarts from the top; otherwise resume with the usual rewind.
    let saved = match episode.playback_status {
        PlaybackStatus::Played => 0,
        _ => episode.playback_position_secs.unwrap_or(0),
    };
    let start = saved.saturating_sub(RESUME_REWIND_SECS);

    let mut playing_episode = episode.clone();
    playing_episode.playback_status = PlaybackStatus::InProgress;
    playing_episode.playback_position_secs = Some(start);
    sync_episode_row(model, episode_id, &PlaybackStatus::InProgress, Some(start));
    let streaming = matches!(source, PlayerSource::Stream { .. });
    model.active_playback = Some(ActivePlayback {
        episode: playing_episode,
        position_secs: start,
        duration_secs: None,
        is_playing: true,
        source: source.clone(),
        loaded: true,
        last_checkpoint_secs: start,
        error: None,
    });

    let mut cmd = outgoing
        .and(persist_playback(
            episode_id,
            PlaybackStatus::InProgress,
            Some(start),
        ))
        .and(save_play_context(episode_id))
        .and(player_op(
            episode_id,
            PlayerOperation::Load {
                episode_id: episode_id.to_string(),
                source,
                start_secs: start,
                autoplay: true,
            },
        ));
    // Streaming now, local later: kick off the download so the copy is there next
    // time (and for the mid-play swap in `on_download_completed`).
    if streaming {
        cmd = cmd.and(enqueue_download(model, episode_id));
    }
    cmd.and(render())
}

fn save_play_context(episode_id: &str) -> Cmd {
    Command::request_from_shell(StorageOperation::SavePlayContext {
        episode_id: episode_id.to_string(),
    })
    .then_send(|r| Event::PlaybackPersisted(Box::new(r)))
}

/// Resumes the active episode, rewinding slightly for context.
pub(crate) fn play(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if active.is_playing {
        return Command::done();
    }
    active.error = None;
    active.is_playing = true;
    let rewound = active.position_secs.saturating_sub(RESUME_REWIND_SECS);
    active.position_secs = rewound;
    let id = active.episode.id.clone();
    let cmd = if active.loaded {
        player_op(&id, PlayerOperation::Seek { secs: rewound })
            .and(player_op(&id, PlayerOperation::Play))
    } else {
        // Cold-start restore (or recovery after an error): nothing is loaded yet.
        active.loaded = true;
        player_op(
            &id,
            PlayerOperation::Load {
                episode_id: id.clone(),
                source: active.source.clone(),
                start_secs: rewound,
                autoplay: true,
            },
        )
    };
    cmd.and(render())
}

pub(crate) fn pause(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if !active.is_playing {
        return Command::done();
    }
    active.is_playing = false;
    let id = active.episode.id.clone();
    player_op(&id, PlayerOperation::Pause)
        .and(checkpoint(model))
        .and(render())
}

pub(crate) fn toggle(model: &mut Model) -> Cmd {
    match model.active_playback.as_ref() {
        Some(a) if a.is_playing => pause(model),
        Some(_) => play(model),
        None => Command::done(),
    }
}

/// Moves the playhead, clamped to the episode: a skip never runs past the end (so it
/// can't spill into another episode) or before the start.
pub(crate) fn seek_to(model: &mut Model, secs: u32) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    let duration = active.duration_secs.or(active.episode.duration_secs);
    let target = duration.map_or(secs, |d| secs.min(d));
    active.position_secs = target;
    let id = active.episode.id.clone();
    let op = if active.loaded {
        PlayerOperation::Seek { secs: target }
    } else {
        // Seeking a restored-but-unloaded episode loads it paused at the new spot.
        active.loaded = true;
        PlayerOperation::Load {
            episode_id: id.clone(),
            source: active.source.clone(),
            start_secs: target,
            autoplay: false,
        }
    };
    player_op(&id, op).and(checkpoint(model)).and(render())
}

pub(crate) fn skip_forward(model: &mut Model) -> Cmd {
    let Some(position) = model.active_playback.as_ref().map(|a| a.position_secs) else {
        return Command::done();
    };
    seek_to(model, position.saturating_add(SKIP_FORWARD_SECS))
}

pub(crate) fn skip_back(model: &mut Model) -> Cmd {
    let Some(position) = model.active_playback.as_ref().map(|a| a.position_secs) else {
        return Command::done();
    };
    seek_to(model, position.saturating_sub(SKIP_BACKWARD_SECS))
}

pub(crate) fn on_tick(model: &mut Model, episode_id: &str, position_secs: u32) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if active.episode.id != episode_id {
        return Command::done();
    }
    active.position_secs = position_secs;
    let due = active.is_playing
        && position_secs.abs_diff(active.last_checkpoint_secs) >= POSITION_CHECKPOINT_SECS;
    if due {
        checkpoint(model).and(render())
    } else {
        render()
    }
}

pub(crate) fn on_duration(model: &mut Model, episode_id: &str, duration_secs: u32) -> Cmd {
    match model.active_playback.as_mut() {
        Some(active) if active.episode.id == episode_id => {
            active.duration_secs = Some(duration_secs);
            render()
        }
        _ => Command::done(),
    }
}

/// The engine reached the end of the file: the episode is played and the player
/// goes inactive (no auto-advance until playlists exist).
pub(crate) fn on_ended(model: &mut Model, episode_id: &str) -> Cmd {
    if model
        .active_playback
        .as_ref()
        .is_none_or(|a| a.episode.id != episode_id)
    {
        return Command::done();
    }
    model.active_playback = None;
    sync_episode_row(model, episode_id, &PlaybackStatus::Played, Some(0));
    persist_playback(episode_id, PlaybackStatus::Played, Some(0))
        .and(
            Command::request_from_shell(StorageOperation::ClearPlayContext)
                .then_send(|r| Event::PlaybackPersisted(Box::new(r))),
        )
        .and(player_op(episode_id, PlayerOperation::Stop))
        .and(render())
}

/// The engine couldn't play (or lost) the item. A missing local file falls back to
/// streaming and re-downloads; anything else leaves playback paused with a notice.
pub(crate) fn on_failure(model: &mut Model, episode_id: &str, message: String) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if active.episode.id != episode_id {
        return Command::done();
    }
    let was_playing = active.is_playing;
    let position = active.position_secs;
    let id = active.episode.id.clone();
    if !matches!(active.source, PlayerSource::Local { .. }) {
        active.is_playing = false;
        active.loaded = false;
        active.error = Some(message);
        return checkpoint(model).and(render());
    }
    // The downloaded file is unreadable (deleted behind our back, say): forget it and
    // stream instead, carrying on from the same spot.
    let source = PlayerSource::Stream {
        url: active.episode.enclosure_url.clone(),
    };
    active.source = source.clone();
    active.episode.download_status = DownloadStatus::NotDownloaded;
    active.episode.local_path = None;
    active.error = None;
    active.loaded = true;
    reset_to_not_downloaded(model, &id)
        .and(enqueue_download(model, &id))
        .and(player_op(
            &id,
            PlayerOperation::Load {
                episode_id: id.clone(),
                source,
                start_secs: position,
                autoplay: was_playing,
            },
        ))
        .and(render())
}

pub(crate) fn on_response(model: &mut Model, episode_id: &str, result: PlayerResult) -> Cmd {
    match result {
        PlayerResult::Ok => Command::done(),
        PlayerResult::Error(message) => on_failure(model, episode_id, message),
    }
}

/// A download finished: if it's the episode on the player, move playback onto the
/// local file at the current position, so streaming stops using the network.
pub(crate) fn on_download_completed(
    model: &mut Model,
    episode_id: &str,
    local_path: &str,
    size_bytes: u64,
) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if active.episode.id != episode_id {
        return Command::done();
    }
    active.episode.download_status = DownloadStatus::Downloaded;
    active.episode.local_path = Some(local_path.to_string());
    active.episode.file_size_bytes = Some(size_bytes);
    if !matches!(active.source, PlayerSource::Stream { .. }) {
        return Command::done();
    }
    let source = PlayerSource::Local {
        local_path: local_path.to_string(),
    };
    active.source = source.clone();
    if !active.loaded {
        return render();
    }
    player_op(
        episode_id,
        PlayerOperation::Load {
            episode_id: episode_id.to_string(),
            source,
            start_secs: active.position_secs,
            autoplay: active.is_playing,
        },
    )
}

/// Restores the active episode at launch, paused, so the mini-player is there
/// immediately. The engine isn't loaded until the first Play.
pub(crate) fn on_context_loaded(model: &mut Model, result: StorageResult) -> Cmd {
    if model.active_playback.is_some() {
        return Command::done();
    }
    let StorageResult::Episode(episode) = result else {
        return Command::done();
    };
    let position = match episode.playback_status {
        PlaybackStatus::Played => 0,
        _ => episode.playback_position_secs.unwrap_or(0),
    };
    model.active_playback = Some(ActivePlayback {
        source: source_for(&episode),
        episode,
        position_secs: position,
        duration_secs: None,
        is_playing: false,
        loaded: false,
        last_checkpoint_secs: position,
        error: None,
    });
    render()
}

/// Flushes position when the app leaves the foreground.
pub(crate) fn on_backgrounded(model: &mut Model) -> Cmd {
    checkpoint(model)
}

pub(crate) fn player_view(model: &Model) -> Option<PlayerView> {
    let active = model.active_playback.as_ref()?;
    let subscription = model
        .subscriptions
        .iter()
        .find(|s| s.id == active.episode.subscription_id);
    Some(PlayerView {
        episode_id: active.episode.id.clone(),
        subscription_id: active.episode.subscription_id.clone(),
        episode_title: active.episode.title.clone(),
        feed_title: subscription.map(|s| s.title.clone()).unwrap_or_default(),
        artwork_url: active
            .episode
            .artwork_url
            .clone()
            .or_else(|| subscription.and_then(|s| s.artwork_url.clone())),
        position_secs: active.position_secs,
        duration_secs: active.duration_secs.or(active.episode.duration_secs),
        is_playing: active.is_playing,
        is_streaming: matches!(active.source, PlayerSource::Stream { .. }),
        skip_forward_secs: SKIP_FORWARD_SECS,
        skip_back_secs: SKIP_BACKWARD_SECS,
        error: active.error.clone(),
    })
}
